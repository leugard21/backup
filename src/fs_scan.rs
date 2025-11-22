use crate::types::FileEntry;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

use crate::filter::PathFilter;

fn is_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file()
}

pub fn scan_dir(root: &Path, filter: Option<&PathFilter>) -> Vec<FileEntry> {
    let mut out = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_file(e))
    {
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or_else(|_| entry.path());

        if let Some(f) = filter {
            if !f.allow(rel) {
                continue;
            }
        }

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
