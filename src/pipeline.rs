use indicatif::ProgressBar;
use rayon::prelude::*;

use crate::hasher::hash_file;
use crate::types::FileEntry;

#[derive(Debug, Clone)]
pub struct HashedFile {
    pub entry: FileEntry,
    pub hash: [u8; 32],
}

pub fn hash_files_parallel(files: &[FileEntry], pb: &ProgressBar) -> Vec<HashedFile> {
    pb.set_length(files.len() as u64);

    let result: Vec<HashedFile> = files
        .par_iter()
        .filter_map(|f| {
            let hashed = hash_file(&f.path).map(|h| HashedFile {
                entry: f.clone(),
                hash: h,
            });

            pb.inc(1);
            hashed
        })
        .collect();

    pb.finish_with_message("hashing complete");
    result
}
