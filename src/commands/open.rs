use anyhow::{bail, Context, Result};
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::path::Path;

use super::secure_delete::secure_delete;

pub fn open(file: &Path, identity: Option<&Path>) -> Result<()> {
    if !file.exists() {
        bail!("File not found: {}", file.display());
    }
    if file.extension().map(|e| e != "age").unwrap_or(true) {
        bail!("Not an .age file: {}", file.display());
    }

    // Determine output filename (strip .age)
    let stem = file.with_extension("");
    let tmp_dir = tempfile::tempdir().context("Cannot create temp directory")?;
    let output_path = tmp_dir.path().join(
        stem.file_name().unwrap_or_default()
    );

    // Decrypt
    let encrypted = fs::read(file).context("Cannot read encrypted file")?;
    let decrypted = if let Some(id_path) = identity {
        decrypt_with_identity(&encrypted, id_path)?
    } else {
        decrypt_with_passphrase(&encrypted)?
    };

    fs::write(&output_path, &decrypted)
        .with_context(|| format!("Cannot write: {}", output_path.display()))?;

    eprintln!("✓ Decrypted to: {}", output_path.display());
    eprintln!("Press Enter when done (plaintext will be securely deleted)...");
    io::stderr().flush()?;
    let _ = io::stdin().lock().lines().next();

    // Securely delete plaintext
    secure_delete(&output_path)?;
    eprintln!("✓ Plaintext securely deleted");
    Ok(())
}

fn decrypt_with_identity(data: &[u8], id_path: &Path) -> Result<Vec<u8>> {
    let id_str = fs::read_to_string(id_path)
        .with_context(|| format!("Cannot read identity: {}", id_path.display()))?;
    let identities: Vec<age::x25519::Identity> = age::IdentityFile::from_buffer(id_str.as_bytes())
        .map_err(|e| anyhow::anyhow!("Invalid identity file: {}", e))?
        .into_identities()
        .into_iter()
        .filter_map(|entry| match entry {
            age::IdentityFileEntry::Native(id) => Some(id),
        })
        .collect();

    let decryptor = match age::Decryptor::new(data)
        .map_err(|e| anyhow::anyhow!("Decryption error: {}", e))? {
        age::Decryptor::Recipients(d) => d,
        _ => bail!("File was encrypted with passphrase, not key"),
    };

    let mut reader = decryptor.decrypt(identities.iter().map(|i| i as &dyn age::Identity))
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}

fn decrypt_with_passphrase(data: &[u8]) -> Result<Vec<u8>> {
    eprint!("Passphrase: ");
    io::stderr().flush()?;
    let pass = io::stdin().lock().lines().next()
        .ok_or_else(|| anyhow::anyhow!("No passphrase"))??;

    let decryptor = match age::Decryptor::new(data)
        .map_err(|e| anyhow::anyhow!("Decryption error: {}", e))? {
        age::Decryptor::Passphrase(d) => d,
        _ => bail!("File was encrypted with key, use --identity"),
    };

    let mut reader = decryptor.decrypt(&secrecy::Secret::new(pass), None)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}
