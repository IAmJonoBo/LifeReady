use clap::{Parser, Subcommand};
use std::path::PathBuf;

use audit_verifier::{verify_audit_chain, verify_bundle, verify_manifest};

#[derive(Debug, Parser)]
#[command(name = "audit-verifier")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::enum_variant_names)]
enum Command {
    VerifyAudit {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        head_hash: Option<String>,
    },
    VerifyManifest {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        bundle_dir: Option<PathBuf>,
    },
    VerifyBundle {
        #[arg(long)]
        bundle: PathBuf,
    },
}

fn run(args: Args) -> Result<(), String> {
    

    match args.command {
        Command::VerifyAudit { input, head_hash } => {
            verify_audit_chain(&input, head_hash.as_deref()).map(|head| {
                println!("Audit chain OK. Head hash: {}", head);
            })
        }
        Command::VerifyManifest {
            manifest,
            bundle_dir,
        } => verify_manifest(&manifest, bundle_dir.as_deref()).map(|_| {
            println!("Manifest OK.");
        }),
        Command::VerifyBundle { bundle } => verify_bundle(&bundle).map(|_| {
            println!("Bundle OK.");
        }),
    }
}

fn main() {
    let args = Args::parse();
    if let Err(error) = run(args) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{name}-{}-{}", std::process::id(), nanos))
    }

    #[test]
    fn run_verify_audit_accepts_empty_file() {
        let dir = unique_dir("audit-cli");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("audit.jsonl");
        fs::write(&path, "").unwrap();

        let args = Args {
            command: Command::VerifyAudit {
                input: path,
                head_hash: None,
            },
        };
        run(args).expect("verify audit");
    }

    #[test]
    fn run_verify_manifest_accepts_valid_manifest() {
        let dir = unique_dir("audit-cli");
        fs::create_dir_all(&dir).unwrap();
        let manifest_path = dir.join("manifest.json");
        let manifest = serde_json::json!({
            "case_id": "case-1",
            "case_type": "mhca39",
            "exported_at": "2025-01-01T00:00:00Z",
            "audit_head_hash": "",
            "audit_events_sha256": "a".repeat(64),
            "documents": []
        });
        fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

        let args = Args {
            command: Command::VerifyManifest {
                manifest: manifest_path,
                bundle_dir: None,
            },
        };
        run(args).expect("verify manifest");
    }

    #[test]
    fn run_verify_bundle_accepts_manifest_only_bundle() {
        let dir = unique_dir("audit-cli");
        fs::create_dir_all(&dir).unwrap();
        let manifest_path = dir.join("manifest.json");
        let manifest = serde_json::json!({
            "case_id": "case-1",
            "case_type": "mhca39",
            "exported_at": "2025-01-01T00:00:00Z",
            "audit_head_hash": "",
            "audit_events_sha256": "a".repeat(64),
            "documents": []
        });
        fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

        let args = Args {
            command: Command::VerifyBundle { bundle: dir },
        };
        run(args).expect("verify bundle");
    }

    #[test]
    fn run_returns_error_on_head_mismatch() {
        let dir = unique_dir("audit-cli");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("audit.jsonl");
        fs::write(&path, "").unwrap();

        let args = Args {
            command: Command::VerifyAudit {
                input: path,
                head_hash: Some("bad".to_string()),
            },
        };
        assert!(run(args).is_err());
    }
}
