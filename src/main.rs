mod backup_file;
mod config;
mod fs_scan;
mod hasher;
mod inspect;
mod manifest;
mod pipeline;
mod restore;
mod types;
mod validation;

use indicatif::ProgressBar;
use pipeline::hash_files_parallel;
use rayon::ThreadPoolBuilder;
use std::env;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::BackupConfig;
use crate::manifest::build_manifest_json;
use crate::validation::validate_paths;

fn main() {
    let mut args = env::args().skip(1);
    let Some(first) = args.next() else {
        eprintln!("usage:");
        eprintln!("  backup <source-dir> <backup-dir> [--threads N] [--verify]");
        eprintln!("  backup inspect <backup-file>");
        eprintln!("  backup restore <backup-file> <restore-dir>");
        return;
    };

    if first == "inspect" {
        let Some(archive) = args.next() else {
            eprintln!("usage: backup inspect <backup-file>");
            return;
        };

        if let Err(e) = inspect::inspect_backup(Path::new(&archive)) {
            eprintln!("inspect failed: {e}");
        }
        return;
    }

    if first == "restore" {
        let Some(archive) = args.next() else {
            eprintln!("usage: backup restore <backup-file> <restore-dir>");
            return;
        };
        let Some(dest) = args.next() else {
            eprintln!("usage: backup restore <backup-file> <restore-dir>");
            return;
        };

        if let Err(e) = restore::restore_backup(Path::new(&archive), Path::new(&dest)) {
            eprintln!("restore failed: {e}");
        }
        return;
    }

    let config = match BackupConfig::from_args(first, args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("usage: backup <source-dir> <backup-dir> [--threads N] [--verify]");
            eprintln!("error: {e}");
            return;
        }
    };

    if let Some(n) = config.threads {
        if let Err(err) = ThreadPoolBuilder::new().num_threads(n).build_global() {
            eprintln!("warning: failed to configure thread pool: {err}");
        }
    }

    let paths = match validate_paths(&config) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("configuration error: {e}");
            return;
        }
    };

    let source_name = paths
        .source_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("backup");

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let backup_file_name = format!("{source_name}-{ts}.backup");
    let backup_file = paths.backup_dir.join(&backup_file_name);

    println!("source: {:?}", paths.source_root);
    println!("backup dir: {:?}", paths.backup_dir);
    println!("backup file: {:?}", backup_file);

    println!("scanning: {:?}", paths.source_root);
    let files = fs_scan::scan_dir(&paths.source_root);
    println!("scanned: {} files", files.len());

    if files.is_empty() {
        println!("nothing to hash or backup");
        return;
    }

    let pb_hash = ProgressBar::new(files.len() as u64);
    let hashed = hash_files_parallel(&files, &pb_hash);
    println!("hashed: {} files", hashed.len());

    println!("building manifest...");
    let manifest_json = match build_manifest_json(&paths.source_root, &backup_file, &hashed) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("failed to build manifest JSON: {e}");
            return;
        }
    };

    println!("writing backup archive...");
    let pb_backup = ProgressBar::new(0);
    if let Err(e) = backup_file::create_backup_file(
        &backup_file,
        &paths.source_root,
        &hashed,
        &manifest_json,
        &pb_backup,
    ) {
        eprintln!("failed to create backup file: {e}");
        return;
    }
    println!("backup written to: {:?}", backup_file);

    if config.verify {
        eprintln!("warning: --verify for .backup archives is not implemented yet and was ignored");
    }
}
