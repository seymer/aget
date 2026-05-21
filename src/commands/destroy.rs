use anyhow::{bail, Result};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use super::secure_delete::secure_delete;

pub fn destroy(files: &[PathBuf], no_confirm: bool, passes: u32) -> Result<()> {
    if files.is_empty() {
        bail!("No files specified");
    }

    if !no_confirm {
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
    }

    for file in files {
        if !file.exists() {
            eprintln!("  ⚠ Skipping (not found): {}", file.display());
            continue;
        }
        secure_delete(file, passes)?;
        eprintln!("  ✓ Destroyed: {}", file.display());
    }
    Ok(())
}
