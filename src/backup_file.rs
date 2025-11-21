use crate::pipeline::HashedFile;
use indicatif::ProgressBar;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

pub fn create_backup_file(
    backup_file: &Path,
    source_root: &Path,
    files: &[HashedFile],
    manifest_json: &str,
    pb: &ProgressBar,
) -> io::Result<()> {
    let file = File::create(backup_file)?;
    let mut writer = BufWriter::new(file);

    writer.write_all(b"BKUP")?;
    writer.write_all(&1u32.to_le_bytes())?;

    let manifest_bytes = manifest_json.as_bytes();
    let manifest_len = manifest_bytes.len() as u64;
    writer.write_all(&manifest_len.to_le_bytes())?;
    writer.write_all(manifest_bytes)?;

    let total_bytes: u64 = files.iter().map(|h| h.entry.size).sum();
    pb.set_length(total_bytes);

    let mut buf = [0u8; 8192];

    for h in files {
        let rel = h
            .entry
            .path
            .strip_prefix(source_root)
            .unwrap_or(&h.entry.path);
        let path_str = rel.to_string_lossy();
        let path_bytes = path_str.as_bytes();

        if path_bytes.len() > u16::MAX as usize {
            eprintln!(
                "warning: path too long for backup format, skipping: {}",
                path_str
            );
            continue;
        }

        let path_len = path_bytes.len() as u16;

        writer.write_all(&path_len.to_le_bytes())?;
        writer.write_all(path_bytes)?;
        writer.write_all(&h.entry.size.to_le_bytes())?;
        writer.write_all(&h.hash)?;

        let src_file = match File::open(&h.entry.path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("warning: failed to reopen file {:?}: {e}", h.entry.path);
                continue;
            }
        };

        let mut reader = BufReader::new(src_file);
        let mut remaining = h.entry.size;

        while remaining > 0 {
            let read_len = std::cmp::min(remaining, buf.len() as u64) as usize;
            let n = reader.read(&mut buf[..read_len])?;
            if n == 0 {
                break;
            }
            writer.write_all(&buf[..n])?;
            remaining -= n as u64;
            pb.inc(n as u64);
        }
    }

    writer.flush()?;
    pb.finish_with_message(".backup archive write complete");
    Ok(())
}
