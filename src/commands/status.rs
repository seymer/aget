use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

pub fn status(path: &Path, recursive: bool) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    let mut encrypted = 0u32;
    let mut plaintext = 0u32;

    let walker = if recursive {
        WalkDir::new(path)
    } else {
        WalkDir::new(path).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        let is_age = p.extension().map(|e| e == "age").unwrap_or(false);
        if is_age {
            encrypted += 1;
            println!("  🔒 {}", p.display());
        } else {
            plaintext += 1;
            println!("  📄 {}", p.display());
        }
    }

    println!("\n{} encrypted, {} plaintext", encrypted, plaintext);
    Ok(())
}
