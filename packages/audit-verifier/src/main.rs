use clap::Parser;
use serde::Deserialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Parser)]
#[command(name = "audit-verifier")]
struct Args {
    #[arg(long)]
    input: String,
    #[arg(long)]
    head_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuditAppend {
    actor_principal_id: String,
    action: String,
    tier: String,
    case_id: Option<String>,
    payload: Value,
}

#[derive(Debug, Deserialize)]
struct AuditEvent {
    event_id: String,
    created_at: String,
    prev_hash: String,
    event_hash: String,
    event: AuditAppend,
}

fn main() {
    let args = Args::parse();
    let file = File::open(&args.input).unwrap_or_else(|_| panic!("Failed to open {}", args.input));
    let reader = BufReader::new(file);

    let mut prev_hash = "0".repeat(64);
    let mut last_hash = prev_hash.clone();

    for (idx, line) in reader.lines().enumerate() {
        let line = line.unwrap_or_else(|_| panic!("Failed to read line {}", idx + 1));
        if line.trim().is_empty() {
            continue;
        }
        let event: AuditEvent = serde_json::from_str(&line)
            .unwrap_or_else(|_| panic!("Invalid JSON at line {}", idx + 1));

        if event.prev_hash != prev_hash {
            eprintln!("Chain break at line {}: prev_hash mismatch", idx + 1);
            std::process::exit(1);
        }

        let computed = compute_event_hash(&prev_hash, &event);
        if computed != event.event_hash {
            eprintln!("Hash mismatch at line {}", idx + 1);
            std::process::exit(1);
        }

        prev_hash = event.event_hash.clone();
        last_hash = prev_hash.clone();
    }

    if let Some(expected) = args.head_hash {
        if expected != last_hash {
            eprintln!("Head hash mismatch: expected {}, got {}", expected, last_hash);
            std::process::exit(1);
        }
    }

    println!("Audit chain OK. Head hash: {}", last_hash);
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
