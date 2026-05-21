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
    /// Encrypt file with age and securely delete the original
    Seal {
        /// File to encrypt
        file: PathBuf,
        /// Recipient public key (age1...)
        #[arg(short, long)]
        recipient: Option<String>,
        /// Use passphrase instead of key
        #[arg(short, long)]
        passphrase: bool,
    },
    /// Decrypt .age file, open it, then securely delete plaintext
    Open {
        /// Encrypted .age file
        file: PathBuf,
        /// Identity file for decryption
        #[arg(short, long)]
        identity: Option<PathBuf>,
    },
    /// Securely delete a file
    Destroy {
        /// Files to destroy
        files: Vec<PathBuf>,
    },
    /// Show encryption status of files in a directory
    Status {
        /// Directory to scan (default: current)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Seal { file, recipient, passphrase } => commands::seal(&file, recipient.as_deref(), passphrase),
        Commands::Open { file, identity } => commands::open(&file, identity.as_deref()),
        Commands::Destroy { files } => commands::destroy(&files),
        Commands::Status { path } => commands::status(&path),
    }
}
