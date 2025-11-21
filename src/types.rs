use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
}
