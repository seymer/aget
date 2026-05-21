use anyhow::{bail, Context, Result};
use std::fs::{self, File};
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

use super::secure_delete::secure_delete;

pub fn seal(file: &Path, recipient: Option<&str>, passphrase: bool, keep: bool, passes: u32) -> Result<()> {
    if !file.exists() {
        bail!("File not found: {}", file.display());
    }
    if file.extension().map(|e| e == "age").unwrap_or(false) {
        bail!("File is already encrypted: {}", file.display());
    }

    let mut output_name = file.as_os_str().to_os_string();
    output_name.push(".age");
    let output_path = std::path::PathBuf::from(output_name);

    if passphrase {
        seal_with_passphrase(file, &output_path)?;
    } else if let Some(recipient) = recipient {
        seal_with_recipient(file, &output_path, recipient)?;
    } else {
        bail!("Specify --recipient or --passphrase");
    }

    if keep {
        eprintln!("✓ Sealed: {} (original kept)", output_path.display());
    } else {
        eprintln!("Securely deleting original: {}", file.display());
        secure_delete(file, passes)?;
        eprintln!("✓ Sealed: {}", output_path.display());
    }
    Ok(())
}

fn seal_with_passphrase(input: &Path, output: &Path) -> Result<()> {
    let pass = read_passphrase_with_confirm()?;

    let encryptor = age::Encryptor::with_user_passphrase(secrecy::Secret::new(pass));
    let mut output_file = File::create(output)
        .with_context(|| format!("Cannot create: {}", output.display()))?;
    let mut writer = encryptor.wrap_output(&mut output_file)
        .map_err(|e| anyhow::anyhow!("Encryption error: {}", e))?;
    let mut input_file = File::open(input)
        .with_context(|| format!("Cannot read: {}", input.display()))?;
    io::copy(&mut input_file, &mut writer)?;
    writer.finish()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(output, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn seal_with_recipient(input: &Path, output: &Path, recipient: &str) -> Result<()> {
    let recipient: Box<dyn age::Recipient + Send> = recipient
        .parse::<age::x25519::Recipient>()
        .map(|r| Box::new(r) as Box<dyn age::Recipient + Send>)
        .map_err(|e| anyhow::anyhow!("Invalid recipient: {}", e))?;

    let encryptor = age::Encryptor::with_recipients(vec![recipient])
        .expect("recipients not empty");
    let mut output_file = File::create(output)
        .with_context(|| format!("Cannot create: {}", output.display()))?;
    let mut writer = encryptor.wrap_output(&mut output_file)
        .map_err(|e| anyhow::anyhow!("Encryption error: {}", e))?;
    let mut input_file = File::open(input)
        .with_context(|| format!("Cannot read: {}", input.display()))?;
    io::copy(&mut input_file, &mut writer)?;
    writer.finish()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(output, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn read_passphrase_with_confirm() -> Result<String> {
    let stdin = io::stdin();

    if stdin.is_terminal() {
        eprint!("Passphrase: ");
        io::stderr().flush()?;
        let pass = stdin.lock().lines().next()
            .ok_or_else(|| anyhow::anyhow!("No passphrase provided"))??;
        eprint!("Confirm passphrase: ");
        io::stderr().flush()?;
        let confirm = stdin.lock().lines().next()
            .ok_or_else(|| anyhow::anyhow!("No confirmation"))??;
        if pass != confirm {
            bail!("Passphrases don't match");
        }
        Ok(pass)
    } else {
        let line = stdin.lock().lines().next()
            .ok_or_else(|| anyhow::anyhow!("No passphrase provided"))??;
        Ok(line)
    }
}
