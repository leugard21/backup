use crate::types::FileEntry;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::fs;
use std::path::Path;

pub fn copy_files_parallel(
    files: &[FileEntry],
    source_root: &Path,
    dest_root: &Path,
    pb: &ProgressBar,
) -> usize {
    pb.set_length(files.len() as u64);

    let copied_count: usize = files
        .par_iter()
        .map(|f| {
            let rel = match f.path.strip_prefix(source_root) {
                Ok(r) => r,
                Err(_) => {
                    eprintln!(
                        "warning: path {:?} is not under source root {:?}, skipping",
                        f.path, source_root
                    );
                    pb.inc(1);
                    return 0usize;
                }
            };

            let dest_path = dest_root.join(rel);

            if let Some(parent) = dest_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("failed to create dir {:?}: {e}", parent);
                    pb.inc(1);
                    return 0usize;
                }
            }

            match fs::copy(&f.path, &dest_path) {
                Ok(_) => {
                    pb.inc(1);
                    1
                }
                Err(e) => {
                    eprintln!("failed to copy {:?} -> {:?}: {e}", f.path, dest_path);
                    pb.inc(1);
                    0
                }
            }
        })
        .sum();

    pb.finish_with_message("copy complete");
    copied_count
}
