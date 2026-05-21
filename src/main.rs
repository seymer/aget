mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aget", about = "Secure file encryption and destruction")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt file with age (original securely deleted by default)
    Seal {
        /// File to encrypt
        file: PathBuf,
        /// Recipient public key (age1...)
        #[arg(short, long)]
        recipient: Option<String>,
        /// Use passphrase instead of key
        #[arg(short, long)]
        passphrase: bool,
        /// Keep the original file (don't delete)
        #[arg(short, long)]
        keep: bool,
        /// Number of overwrite passes for secure deletion (default: 3)
        #[arg(long, default_value = "3")]
        passes: u32,
    },
    /// Decrypt .age file
    Open {
        /// Encrypted .age file
        file: PathBuf,
        /// Identity file for decryption
        #[arg(short, long)]
        identity: Option<PathBuf>,
        /// Output directory (default: temp dir with auto-cleanup)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Securely delete files
    Destroy {
        /// Files to destroy
        files: Vec<PathBuf>,
        /// Skip confirmation prompt
        #[arg(long)]
        no_confirm: bool,
        /// Number of overwrite passes (default: 3)
        #[arg(long, default_value = "3")]
        passes: u32,
    },
    /// Show encryption status of files in a directory
    Status {
        /// Directory to scan (default: current)
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Scan recursively
        #[arg(short, long)]
        recursive: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Seal { file, recipient, passphrase, keep, passes } => {
            commands::seal(&file, recipient.as_deref(), passphrase, keep, passes)
        }
        Commands::Open { file, identity, output } => {
            commands::open(&file, identity.as_deref(), output.as_deref())
        }
        Commands::Destroy { files, no_confirm, passes } => {
            commands::destroy(&files, no_confirm, passes)
        }
        Commands::Status { path, recursive } => {
            commands::status(&path, recursive)
        }
    }
}
