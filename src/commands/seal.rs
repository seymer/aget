use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;

use super::secure_delete::secure_delete;

pub fn seal(file: &Path, recipient: Option<&str>, passphrase: bool) -> Result<()> {
    if !file.exists() {
        bail!("File not found: {}", file.display());
    }
    if file.extension().map(|e| e == "age").unwrap_or(false) {
        bail!("File is already encrypted: {}", file.display());
    }

    let output_path = file.with_extension(
        format!("{}.age", file.extension().unwrap_or_default().to_string_lossy())
    );

    if passphrase {
        seal_with_passphrase(file, &output_path)?;
    } else if let Some(recipient) = recipient {
        seal_with_recipient(file, &output_path, recipient)?;
    } else {
        bail!("Specify --recipient or --passphrase");
    }

    // Securely delete original
    eprintln!("Securely deleting original: {}", file.display());
    secure_delete(file)?;
    eprintln!("✓ Sealed: {}", output_path.display());
    Ok(())
}

fn seal_with_passphrase(input: &Path, output: &Path, ) -> Result<()> {
    eprint!("Passphrase: ");
    io::stderr().flush()?;
    let pass = read_passphrase()?;

    let encryptor = age::Encryptor::with_user_passphrase(secrecy::Secret::new(pass));
    let input_data = std::fs::read(input)
        .with_context(|| format!("Cannot read: {}", input.display()))?;

    let mut output_file = File::create(output)
        .with_context(|| format!("Cannot create: {}", output.display()))?;
    let mut writer = encryptor.wrap_output(&mut output_file)
        .map_err(|e| anyhow::anyhow!("Encryption error: {}", e))?;
    writer.write_all(&input_data)?;
    writer.finish()?;
    Ok(())
}

fn seal_with_recipient(input: &Path, output: &Path, recipient: &str) -> Result<()> {
    let recipient: Box<dyn age::Recipient + Send> = recipient
        .parse::<age::x25519::Recipient>()
        .map(|r| Box::new(r) as Box<dyn age::Recipient + Send>)
        .map_err(|e| anyhow::anyhow!("Invalid recipient: {}", e))?;

    let encryptor = age::Encryptor::with_recipients(vec![recipient])
        .expect("recipients not empty");
    let input_data = std::fs::read(input)
        .with_context(|| format!("Cannot read: {}", input.display()))?;

    let mut output_file = File::create(output)
        .with_context(|| format!("Cannot create: {}", output.display()))?;
    let mut writer = encryptor.wrap_output(&mut output_file)
        .map_err(|e| anyhow::anyhow!("Encryption error: {}", e))?;
    writer.write_all(&input_data)?;
    writer.finish()?;
    Ok(())
}

fn read_passphrase() -> Result<String> {
    let stdin = io::stdin();
    let line = stdin.lock().lines().next()
        .ok_or_else(|| anyhow::anyhow!("No passphrase provided"))??;
    Ok(line)
}
