use crate::hasher::hash_file;
use crate::pipeline::HashedFile;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct VerifySummary {
    pub checked: usize,
    pub ok: usize,
    pub missing: usize,
    pub mismatched: usize,
}

pub fn verify_copied_files(
    hashed: &[HashedFile],
    source_root: &Path,
    dest_root: &Path,
    pb: &ProgressBar,
) -> VerifySummary {
    pb.set_length(hashed.len() as u64);

    let (checked, ok, missing, mismatched) = hashed
        .par_iter()
        .map(|h| {
            let rel = h
                .entry
                .path
                .strip_prefix(source_root)
                .unwrap_or(&h.entry.path);
            let dest_path = dest_root.join(rel);

            let checked = 1usize;
            let mut ok = 0usize;
            let mut missing = 0usize;
            let mut mismatched = 0usize;

            if !dest_root.exists() {
                missing = 1;
            } else {
                match hash_file(&dest_path) {
                    Some(hash) if hash == h.hash => {
                        ok = 1;
                    }
                    Some(_) => {
                        mismatched = 1;
                    }
                    None => {
                        mismatched = 1;
                    }
                }
            }

            pb.inc(1);
            (checked, ok, missing, mismatched)
        })
        .reduce(
            || (0usize, 0usize, 0usize, 0usize),
            |a, b| (a.0 + b.0, a.1 + b.1, a.2 + b.2, a.3 + b.3),
        );

    pb.finish_with_message("verify complete");

    VerifySummary {
        checked,
        ok,
        missing,
        mismatched,
    }
}
