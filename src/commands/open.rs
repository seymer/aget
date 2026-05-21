use anyhow::{bail, Context, Result};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

use super::secure_delete::secure_delete;

pub fn open(file: &Path, identity: Option<&Path>, output: Option<&Path>) -> Result<()> {
    if !file.exists() {
        bail!("File not found: {}", file.display());
    }
    if file.extension().map(|e| e != "age").unwrap_or(true) {
        bail!("Not an .age file: {}", file.display());
    }

    // Determine output filename (strip .age)
    let stem = file.with_extension("");
    let filename = stem.file_name().unwrap_or_default();

    if let Some(out_dir) = output {
        // --output mode: decrypt to specified directory, no auto-cleanup
        if !out_dir.exists() {
            fs::create_dir_all(out_dir)?;
        }
        let output_path = out_dir.join(filename);
        decrypt_to(file, identity, &output_path)?;
        println!("{}", output_path.display());
    } else {
        // Default: decrypt to temp dir, wait for user, then securely delete
        let tmp_dir = tempfile::tempdir().context("Cannot create temp directory")?;
        let output_path = tmp_dir.path().join(filename);
        decrypt_to(file, identity, &output_path)?;

        eprintln!("✓ Decrypted to: {}", output_path.display());
        eprintln!("Press Enter when done (plaintext will be securely deleted)...");
        io::stderr().flush()?;
        let _ = io::stdin().lock().lines().next();

        secure_delete(&output_path, 3)?;
        eprintln!("✓ Plaintext securely deleted");
    }
    Ok(())
}

fn decrypt_to(file: &Path, identity: Option<&Path>, output_path: &Path) -> Result<()> {
    let encrypted_file = fs::File::open(file).context("Cannot read encrypted file")?;

    if let Some(id_path) = identity {
        decrypt_with_identity(encrypted_file, id_path, output_path)?;
    } else {
        decrypt_with_passphrase(encrypted_file, output_path)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(output_path, fs::Permissions::from_mode(0o600))?;
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
        .map(|entry| match entry {
            age::IdentityFileEntry::Native(id) => id,
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
    eprint!("Passphrase: ");
    io::stderr().flush()?;
    let pass = io::stdin().lock().lines().next()
        .ok_or_else(|| anyhow::anyhow!("No passphrase"))??;

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
