use crate::pipeline::HashedFile;
use serde::Serialize;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize)]
struct ManifestFile {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Serialize)]
struct BackupManifest {
    pub source: String,
    pub destination: String,
    pub created_at: u64,
    pub files: Vec<ManifestFile>,
}

fn hash_to_hex(hash: &[u8; 32]) -> String {
    use std::fmt::Write as FmtWrite;

    let mut s = String::with_capacity(64);
    for b in hash {
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}

pub fn write_manifest(
    source_root: &Path,
    dest_root: &Path,
    hashed: &[HashedFile],
    output_path: Option<&Path>,
) -> std::io::Result<PathBuf> {
    let files: Vec<ManifestFile> = hashed
        .iter()
        .map(|h| {
            let rel = h
                .entry
                .path
                .strip_prefix(source_root)
                .unwrap_or(&h.entry.path);

            ManifestFile {
                path: rel.to_string_lossy().to_string(),
                size: h.entry.size,
                sha256: hash_to_hex(&h.hash),
            }
        })
        .collect();

    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let manifest = BackupManifest {
        source: source_root.to_string_lossy().to_string(),
        destination: dest_root.to_string_lossy().to_string(),
        created_at,
        files,
    };

    let path = if let Some(p) = output_path {
        p.to_path_buf()
    } else {
        dest_root.join("backup-manifest.json")
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(&path)?;
    serde_json::to_writer_pretty(file, &manifest)?;

    Ok(path)
}
