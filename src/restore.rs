use indicatif::ProgressBar;
use ring::digest;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ManifestFile {
    pub size: u64,
}

#[derive(Debug, Deserialize)]
struct BackupManifest {
    pub source: String,
    pub files: Vec<ManifestFile>,
}

fn print_section(title: &str) {
    println!();
    println!("--- {title} ---");
}

fn print_kv<K: AsRef<str>, V: AsRef<str>>(k: K, v: V) {
    println!("  {:12} {}", format!("{}:", k.as_ref()), v.as_ref());
}

pub fn restore_backup(backup_file: &Path, restore_dir: &Path) -> io::Result<()> {
    println!("==================== backup restore ====================");
    print_kv("archive", &backup_file.to_string_lossy());
    print_kv("target", &restore_dir.to_string_lossy());

    let file = File::open(backup_file)?;
    let mut reader = BufReader::new(file);

    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != b"BKUP" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid magic, not a .backup file",
        ));
    }

    let mut ver_bytes = [0u8; 4];
    reader.read_exact(&mut ver_bytes)?;
    let version = u32::from_le_bytes(ver_bytes);
    if version != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported backup version: {version}"),
        ));
    }

    let mut len_bytes = [0u8; 8];
    reader.read_exact(&mut len_bytes)?;
    let manifest_len = u64::from_le_bytes(len_bytes) as usize;

    let mut manifest_bytes = vec![0u8; manifest_len];
    reader.read_exact(&mut manifest_bytes)?;

    let manifest: BackupManifest = serde_json::from_slice(&manifest_bytes).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse embedded manifest: {e}"),
        )
    })?;

    let total_bytes: u64 = manifest.files.iter().map(|f| f.size).sum();
    let pb = ProgressBar::new(total_bytes);

    print_section("manifest");
    print_kv("source", &manifest.source);
    print_kv("files", manifest.files.len().to_string());
    print_kv("bytes", total_bytes.to_string());

    if !restore_dir.exists() {
        fs::create_dir_all(restore_dir)?;
    } else if !restore_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "restore destination must be a directory",
        ));
    }

    print_section("restore");
    let mut restored = 0usize;
    let mut mismatched = 0usize;
    let mut failed = 0usize;

    loop {
        let mut len_buf = [0u8; 2];
        match reader.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }

        let path_len = u16::from_le_bytes(len_buf) as usize;
        let mut path_bytes = vec![0u8; path_len];
        reader.read_exact(&mut path_bytes)?;

        let rel_path = match String::from_utf8(path_bytes) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("restore: invalid UTF-8 path in archive: {e}");
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid UTF-8 path in archive",
                ));
            }
        };

        let mut size_buf = [0u8; 8];
        reader.read_exact(&mut size_buf)?;
        let size = u64::from_le_bytes(size_buf);

        let mut expected_hash = [0u8; 32];
        reader.read_exact(&mut expected_hash)?;

        let dest_path = restore_dir.join(&rel_path);

        if let Some(parent) = dest_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("restore: failed to create directory {:?}: {e}", parent);
                if size > 0 {
                    let _ = reader.seek(SeekFrom::Current(size as i64));
                    pb.inc(size);
                }
                failed += 1;
                continue;
            }
        }

        let mut out = match File::create(&dest_path) {
            Ok(f) => BufWriter::new(f),
            Err(e) => {
                eprintln!("restore: failed to create file {:?}: {e}", dest_path);
                if size > 0 {
                    let _ = reader.seek(SeekFrom::Current(size as i64));
                    pb.inc(size);
                }
                failed += 1;
                continue;
            }
        };

        let mut remaining = size;
        let mut buf = [0u8; 8192];
        let mut ctx = digest::Context::new(&digest::SHA256);

        while remaining > 0 {
            let read_len = std::cmp::min(remaining, buf.len() as u64) as usize;
            let n = reader.read(&mut buf[..read_len])?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "truncated file data in archive",
                ));
            }
            out.write_all(&buf[..n])?;
            ctx.update(&buf[..n]);
            remaining -= n as u64;
            pb.inc(n as u64);
        }

        out.flush()?;

        let calc = ctx.finish();
        if calc.as_ref() != expected_hash {
            eprintln!(
                "restore: hash mismatch for {:?} (restored, but contents differ from backup)",
                dest_path
            );
            mismatched += 1;
        } else {
            restored += 1;
        }
    }

    pb.finish_with_message("restore complete");

    print_section("summary");
    print_kv("restored", restored.to_string());
    print_kv("mismatched", mismatched.to_string());
    print_kv("failed", failed.to_string());

    Ok(())
}
