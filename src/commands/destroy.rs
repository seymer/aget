use anyhow::{bail, Result};
use std::io::{self, BufRead, Write};

use super::secure_delete::secure_delete;

pub fn destroy(files: &[std::path::PathBuf]) -> Result<()> {
    if files.is_empty() {
        bail!("No files specified");
    }

    // Confirm
    eprintln!("⚠ Securely delete {} file(s)?", files.len());
    for f in files {
        eprintln!("  {}", f.display());
    }
    eprint!("Type 'yes' to confirm: ");
    io::stderr().flush()?;

    let confirm = io::stdin().lock().lines().next()
        .ok_or_else(|| anyhow::anyhow!("No input"))??;
    if confirm != "yes" {
        eprintln!("Aborted.");
        return Ok(());
    }

    for file in files {
        if !file.exists() {
            eprintln!("  ⚠ Skipping (not found): {}", file.display());
            continue;
        }
        secure_delete(file)?;
        eprintln!("  ✓ Destroyed: {}", file.display());
    }
    Ok(())
}
