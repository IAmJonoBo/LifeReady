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

fn main() {
    let args = Args::parse();

    let result = match args.command {
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
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
