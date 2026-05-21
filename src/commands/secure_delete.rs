use anyhow::{Context, Result};
use rand::RngCore;
use std::fs::{self, OpenOptions};
use std::io::{Seek, Write};
use std::path::Path;

const BUF_SIZE: usize = 1024 * 1024; // 1MB buffer

/// Securely delete a file by overwriting with random data then removing.
pub fn secure_delete(path: &Path, passes: u32) -> Result<()> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Cannot access: {}", path.display()))?;
    let len = metadata.len() as usize;

    if len > 0 {
        let mut buf = vec![0u8; len.min(BUF_SIZE)];
        let mut rng = rand::thread_rng();

        let mut file = OpenOptions::new().write(true).open(path)
            .with_context(|| format!("Cannot open for overwrite: {}", path.display()))?;

        // Random passes
        for _ in 0..passes {
            file.seek(std::io::SeekFrom::Start(0))?;
            let mut remaining = len;
            while remaining > 0 {
                let chunk = remaining.min(buf.len());
                rng.fill_bytes(&mut buf[..chunk]);
                file.write_all(&buf[..chunk])?;
                remaining -= chunk;
            }
            file.sync_all()?;
        }

        // Final zero pass (reuse buf, fill with zeros)
        buf.fill(0);
        file.seek(std::io::SeekFrom::Start(0))?;
        let mut remaining = len;
        while remaining > 0 {
            let chunk = remaining.min(buf.len());
            file.write_all(&buf[..chunk])?;
            remaining -= chunk;
        }
        file.sync_all()?;
    }

    fs::remove_file(path)
        .with_context(|| format!("Cannot remove: {}", path.display()))?;
    Ok(())
}
