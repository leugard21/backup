use crate::config::BackupConfig;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ValidatedPaths {
    pub source_root: PathBuf,
    pub backup_dir: PathBuf,
}

pub fn validate_paths(config: &BackupConfig) -> Result<ValidatedPaths, String> {
    let source = &config.source;
    let backup_dir = &config.destination;

    if !source.exists() {
        return Err(format!("source path does not exist: {:?}", source));
    }

    if !source.is_dir() {
        return Err(format!("source must be a directory: {:?}", source));
    }

    if backup_dir.exists() && !backup_dir.is_dir() {
        return Err(format!(
            "backup destination exists and is not a directory: {:?}",
            backup_dir
        ));
    }

    if !backup_dir.exists() {
        fs::create_dir_all(backup_dir)
            .map_err(|e| format!("failed to create backup directory {:?}: {e}", backup_dir))?;
    }

    let source_canon = fs::canonicalize(source)
        .map_err(|e| format!("failed to canonicalize source {:?}: {e}", source))?;
    let backup_dir_canon = fs::canonicalize(backup_dir).map_err(|e| {
        format!(
            "failed to canonicalize backup directory {:?}: {e}",
            backup_dir
        )
    })?;

    if source_canon == backup_dir_canon {
        return Err("backup directory cannot be the same as source directory".to_string());
    }

    if backup_dir_canon.starts_with(&source_canon) {
        return Err("backup directory cannot be inside source directory".to_string());
    }

    Ok(ValidatedPaths {
        source_root: source_canon,
        backup_dir: backup_dir_canon,
    })
}
