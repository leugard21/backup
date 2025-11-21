use crate::pipeline::HashedFile;
use serde::Serialize;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct ManifestFile {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}
#[derive(Serialize)]
struct BackupManifest {
    pub source: String,
    pub backup_file: String,
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

pub fn build_manifest_json(
    source_root: &Path,
    backup_file: &Path,
    hashed: &[HashedFile],
) -> serde_json::Result<String> {
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
        backup_file: backup_file.to_string_lossy().to_string(),
        created_at,
        files,
    };

    serde_json::to_string_pretty(&manifest)
}
