use serde::Deserialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct AuditAppend {
    pub actor_principal_id: String,
    pub action: String,
    pub tier: String,
    pub case_id: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub created_at: String,
    pub prev_hash: String,
    pub event_hash: String,
    pub event: AuditAppend,
}

#[derive(Debug, Deserialize)]
pub struct ExportManifest {
    pub case_id: String,
    pub case_type: String,
    pub exported_at: String,
    pub audit_head_hash: String,
    pub audit_events_sha256: String,
    pub documents: Vec<ManifestDocument>,
}

#[derive(Debug, Deserialize)]
pub struct ManifestDocument {
    pub slot_name: String,
    pub document_id: String,
    pub document_type: String,
    pub title: String,
    pub sha256: String,
    pub bundle_path: String,
}

pub fn verify_audit_chain(input: &Path, expected_head: Option<&str>) -> Result<String, String> {
    let file = fs::File::open(input)
        .map_err(|error| format!("Failed to open {}: {error}", input.display()))?;
    let reader = BufReader::new(file);

    let mut prev_hash = "0".repeat(64);
    let mut last_hash = prev_hash.clone();

    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|error| format!("Failed to read line {}: {error}", idx + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let event: AuditEvent =
            serde_json::from_str(&line).map_err(|_| format!("Invalid JSON at line {}", idx + 1))?;

        if event.prev_hash != prev_hash {
            return Err(format!(
                "Chain break at line {}: prev_hash mismatch",
                idx + 1
            ));
        }

        let computed = compute_event_hash(&prev_hash, &event);
        if computed != event.event_hash {
            return Err(format!("Hash mismatch at line {}", idx + 1));
        }

        prev_hash = event.event_hash.clone();
        last_hash = prev_hash.clone();
    }

    if let Some(expected) = expected_head {
        if expected != last_hash {
            return Err(format!(
                "Head hash mismatch: expected {expected}, got {last_hash}"
            ));
        }
    }

    Ok(last_hash)
}

pub fn verify_manifest(manifest_path: &Path, bundle_dir: Option<&Path>) -> Result<(), String> {
    let manifest_bytes =
        fs::read(manifest_path).map_err(|error| format!("Failed to read manifest: {error}"))?;
    let manifest: ExportManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("Invalid manifest JSON: {error}"))?;

    if !manifest.audit_head_hash.is_empty() && manifest.audit_head_hash.len() != 64 {
        return Err("Invalid audit_head_hash in manifest".into());
    }

    if manifest.audit_events_sha256.len() != 64 {
        return Err("Invalid audit_events_sha256 in manifest".into());
    }

    let base_dir = bundle_dir.map(PathBuf::from).unwrap_or_else(|| {
        manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });

    let audit_path = base_dir.join("audit.jsonl");
    if audit_path.exists() {
        let audit_sha = sha256_file(&audit_path)?;
        if audit_sha != manifest.audit_events_sha256 {
            return Err("audit.jsonl checksum mismatch".into());
        }
    }

    for doc in &manifest.documents {
        let path = resolve_bundle_path(&base_dir, &doc.bundle_path)?;
        let sha = sha256_file(&path)?;
        if sha != doc.sha256 {
            return Err(format!("Checksum mismatch for {}", doc.bundle_path));
        }
    }

    Ok(())
}

pub fn verify_bundle(bundle_dir: &Path) -> Result<(), String> {
    let manifest_path = bundle_dir.join("manifest.json");
    let manifest_bytes =
        fs::read(&manifest_path).map_err(|error| format!("Failed to read manifest: {error}"))?;
    let manifest: ExportManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("Invalid manifest JSON: {error}"))?;

    verify_manifest(&manifest_path, Some(bundle_dir))?;

    let audit_path = bundle_dir.join("audit.jsonl");
    if audit_path.exists() {
        verify_audit_chain(&audit_path, Some(&manifest.audit_head_hash))?;
    }

    Ok(())
}

fn compute_event_hash(prev_hash: &str, event: &AuditEvent) -> String {
    let canonical = canonical_event_json(event);
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(canonical.as_bytes());
    hex::encode(hasher.finalize())
}

fn canonical_event_json(event: &AuditEvent) -> String {
    let value = serde_json::json!({
        "event_id": event.event_id,
        "created_at": event.created_at,
        "actor_principal_id": event.event.actor_principal_id,
        "action": event.event.action,
        "tier": event.event.tier,
        "case_id": event.event.case_id,
        "payload": event.event.payload,
    });
    let canonical_value = canonicalize_value(&value);
    serde_json::to_string(&canonical_value).unwrap_or_default()
}

fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut ordered = Map::new();
            for key in keys {
                if let Some(inner) = map.get(&key) {
                    ordered.insert(key, canonicalize_value(inner));
                }
            }
            Value::Object(ordered)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_value).collect()),
        _ => value.clone(),
    }
}

fn resolve_bundle_path(base_dir: &Path, bundle_path: &str) -> Result<PathBuf, String> {
    if let Some(path) = bundle_path.strip_prefix("file://") {
        return Ok(PathBuf::from(path));
    }
    if bundle_path.starts_with('/') {
        return Ok(PathBuf::from(bundle_path));
    }
    Ok(base_dir.join(bundle_path))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}
