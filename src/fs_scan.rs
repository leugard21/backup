use crate::types::FileEntry;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

fn is_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file()
}

pub fn scan_dir(root: &Path) -> Vec<FileEntry> {
    let mut out = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_file(e))
    {
        let md = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        out.push(FileEntry {
            path: entry.path().to_path_buf(),
            size: md.len(),
        });
    }

    out
}
