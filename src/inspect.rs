use serde::Deserialize;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ManifestFile {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
struct BackupManifest {
    pub source: String,
    pub backup_file: String,
    pub created_at: u64,
    pub files: Vec<ManifestFile>,
}

pub fn inspect_backup(path: &Path) -> io::Result<()> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != b"BKUP" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid magic, not a .backup file",
        ));
    }

    let mut ver_bytes = [0u8; 4];
    reader.read_exact(&mut ver_bytes)?;
    let version = u32::from_le_bytes(ver_bytes);
    if version != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported backup version: {version}"),
        ));
    }

    let mut len_bytes = [0u8; 8];
    reader.read_exact(&mut len_bytes)?;
    let manifest_len = u64::from_le_bytes(len_bytes) as usize;

    let mut manifest_bytes = vec![0u8; manifest_len];
    reader.read_exact(&mut manifest_bytes)?;

    let manifest: BackupManifest = serde_json::from_slice(&manifest_bytes).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse embedded manifest: {e}"),
        )
    })?;

    let total_files = manifest.files.len();
    let total_bytes: u64 = manifest.files.iter().map(|f| f.size).sum();

    println!("backup info:");
    println!("  source     : {}", manifest.source);
    println!("  backup_file: {}", manifest.backup_file);
    println!("  created_at : {} (unix seconds)", manifest.created_at);
    println!("  files      : {}", total_files);
    println!("  total size : {} bytes", total_bytes);

    println!("  sample files:");
    for f in manifest.files.iter().take(10) {
        println!("    {} ({} bytes)", f.path, f.size);
    }
    if total_files > 10 {
        println!("    ... ({} more files)", total_files - 10);
    }

    Ok(())
}
