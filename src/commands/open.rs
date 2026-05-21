use anyhow::{bail, Context, Result};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

use super::secure_delete::secure_delete;

pub fn open(file: &Path, identity: Option<&Path>, no_wait: bool) -> Result<()> {
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

    // Decrypt using streaming IO
    let encrypted_file = fs::File::open(file).context("Cannot read encrypted file")?;
    if let Some(id_path) = identity {
        decrypt_with_identity(encrypted_file, id_path, &output_path)?;
    } else {
        decrypt_with_passphrase(encrypted_file, &output_path)?;
    }

    // Set restrictive permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&output_path, fs::Permissions::from_mode(0o600))?;
    }

    if no_wait {
        // Print path to stdout for programmatic use, keep temp dir alive by leaking
        println!("{}", output_path.display());
        std::mem::forget(tmp_dir);
        return Ok(());
    }

    eprintln!("✓ Decrypted to: {}", output_path.display());
    eprintln!("Press Enter when done (plaintext will be securely deleted)...");
    io::stderr().flush()?;
    let _ = io::stdin().lock().lines().next();

    // Securely delete plaintext
    secure_delete(&output_path)?;
    eprintln!("✓ Plaintext securely deleted");
    Ok(())
}

/// Clean up a previously decrypted temp file (used by --no-wait callers)
pub fn cleanup(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    secure_delete(path)?;
    // Also remove parent temp dir if empty
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir(parent);
    }
    Ok(())
}

fn decrypt_with_identity(input: fs::File, id_path: &Path, output_path: &Path) -> Result<()> {
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

    let decryptor = match age::Decryptor::new(io::BufReader::new(input))
        .map_err(|e| anyhow::anyhow!("Decryption error: {}", e))? {
        age::Decryptor::Recipients(d) => d,
        _ => bail!("File was encrypted with passphrase, not key"),
    };

    let mut reader = decryptor.decrypt(identities.iter().map(|i| i as &dyn age::Identity))
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    let mut out = fs::File::create(output_path)
        .with_context(|| format!("Cannot write: {}", output_path.display()))?;
    io::copy(&mut reader, &mut out)?;
    Ok(())
}

fn decrypt_with_passphrase(input: fs::File, output_path: &Path) -> Result<()> {
    let pass = if let Ok(p) = std::env::var("AGET_PASSPHRASE") {
        p
    } else {
        eprint!("Passphrase: ");
        io::stderr().flush()?;
        io::stdin().lock().lines().next()
            .ok_or_else(|| anyhow::anyhow!("No passphrase"))??
    };

    let decryptor = match age::Decryptor::new(io::BufReader::new(input))
        .map_err(|e| anyhow::anyhow!("Decryption error: {}", e))? {
        age::Decryptor::Passphrase(d) => d,
        _ => bail!("File was encrypted with key, use --identity"),
    };

    let mut reader = decryptor.decrypt(&secrecy::Secret::new(pass), None)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    let mut out = fs::File::create(output_path)
        .with_context(|| format!("Cannot write: {}", output_path.display()))?;
    io::copy(&mut reader, &mut out)?;
    Ok(())
}
