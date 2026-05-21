use anyhow::{Context, Result};
use rand::RngCore;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

const OVERWRITE_PASSES: usize = 3;

/// Securely delete a file by overwriting with random data then removing.
/// Note: On SSD this is best-effort; encryption is the real protection.
pub fn secure_delete(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Cannot access: {}", path.display()))?;
    let len = metadata.len() as usize;

    if len > 0 {
        let mut buf = vec![0u8; len.min(64 * 1024)];
        for _ in 0..OVERWRITE_PASSES {
            let mut file = OpenOptions::new()
                .write(true)
                .open(path)
                .with_context(|| format!("Cannot open for overwrite: {}", path.display()))?;
            let mut remaining = len;
            while remaining > 0 {
                let chunk = remaining.min(buf.len());
                rand::thread_rng().fill_bytes(&mut buf[..chunk]);
                file.write_all(&buf[..chunk])?;
                remaining -= chunk;
            }
            file.sync_all()?;
        }
        // Final zero pass
        let mut file = OpenOptions::new().write(true).open(path)?;
        let zeros = vec![0u8; len.min(64 * 1024)];
        let mut remaining = len;
        while remaining > 0 {
            let chunk = remaining.min(zeros.len());
            file.write_all(&zeros[..chunk])?;
            remaining -= chunk;
        }
        file.sync_all()?;
    }

    fs::remove_file(path)
        .with_context(|| format!("Cannot remove: {}", path.display()))?;
    Ok(())
}
