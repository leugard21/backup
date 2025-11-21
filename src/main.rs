mod config;
mod copy;
mod fs_scan;
mod hasher;
mod manifest;
mod pipeline;
mod types;
mod validation;
mod verify;

use indicatif::ProgressBar;
use pipeline::hash_files_parallel;
use rayon::ThreadPoolBuilder;

use crate::config::BackupConfig;
use crate::manifest::write_manifest;

fn main() {
    let config = match BackupConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("usage: backup <source> <destination> [--threads N] [--verify]");
            eprintln!("error: {e}");
            return;
        }
    };

    if let Some(n) = config.threads {
        if let Err(err) = ThreadPoolBuilder::new().num_threads(n).build_global() {
            eprintln!("warning: failed to configure thread pool: {err}");
        }
    }

    let (source_root, dest_root) = match validation::validate_paths(&config) {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("configuration error: {e}");
            return;
        }
    };

    println!("source: {:?}", source_root);
    println!("destination: {:?}", dest_root);

    println!("scanning: {:?}", source_root);
    let files = fs_scan::scan_dir(&source_root);
    println!("scanned: {} files", files.len());

    if files.is_empty() {
        println!("nothing to hash or copy");
        return;
    }

    let pb_hash = ProgressBar::new(files.len() as u64);
    let hashed = hash_files_parallel(&files, &pb_hash);
    println!("hashed: {} files", hashed.len());

    println!("writing manifest...");
    match write_manifest(&source_root, &dest_root, &hashed, None) {
        Ok(path) => println!("manifest written to: {:?}", path),
        Err(e) => eprintln!("failed to write manifest: {e}"),
    }

    println!("copying to: {:?}", dest_root);
    let pb_copy = ProgressBar::new(files.len() as u64);
    let copied = copy::copy_files_parallel(&files, &source_root, &dest_root, &pb_copy);
    println!("copied: {} files", copied);

    if config.verify {
        println!("verifying copied files...");
        let pb_verify = ProgressBar::new(hashed.len() as u64);
        let summary = verify::verify_copied_files(&hashed, &source_root, &dest_root, &pb_verify);

        println!(
            "verify: checked={}, ok={}, missing={}, mismatched={}",
            summary.checked, summary.ok, summary.missing, summary.mismatched
        );
    }
}
