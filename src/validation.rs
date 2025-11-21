use crate::config::BackupConfig;
use std::fs;
use std::path::PathBuf;

pub fn validate_paths(config: &BackupConfig) -> Result<(PathBuf, PathBuf), String> {
    let source = &config.source;
    let dest = &config.destination;

    if !source.exists() {
        return Err(format!("source path does not exist: {:?}", source));
    }
    if !source.is_dir() {
        return Err(format!("source must be a directory: {:?}", source));
    }

    if dest.exists() && !dest.is_dir() {
        return Err(format!(
            "destination exists and is not a directory: {:?}",
            dest
        ));
    }

    if !dest.exists() {
        fs::create_dir_all(dest)
            .map_err(|e| format!("failed to create destination directory {:?}: {e}", dest))?;
    }

    let source_canon = fs::canonicalize(source)
        .map_err(|e| format!("failed to canonicalize source {:?}: {e}", source))?;
    let dest_canon = fs::canonicalize(dest)
        .map_err(|e| format!("failed to canonicalize destination {:?}: {e}", dest))?;

    if source_canon == dest_canon {
        return Err("source and destination are the same directory".to_string());
    }

    if dest_canon.starts_with(&source_canon) {
        return Err("destination cannot be inside source directory".to_string());
    }

    Ok((source_canon, dest_canon))
}
