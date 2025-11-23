use indicatif::ProgressBar;
use ring::digest;
use serde::Deserialize;
use std::fs::File;
use std::io::{self, BufReader, Read};
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

pub fn verify_backup_file(path: &Path) -> io::Result<()> {
    println!("==================== backup verify ====================");
    print_kv("archive", &path.to_string_lossy());

    let file = File::open(path)?;
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

    let mut checked = 0usize;
    let mut ok = 0usize;
    let mut mismatched = 0usize;

    print_section("verify");
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

        let _rel_path = match String::from_utf8(path_bytes) {
            Ok(s) => s,
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid UTF-8 path in archive: {e}"),
                ));
            }
        };

        let mut size_buf = [0u8; 8];
        reader.read_exact(&mut size_buf)?;
        let size = u64::from_le_bytes(size_buf);

        let mut expected_hash = [0u8; 32];
        reader.read_exact(&mut expected_hash)?;

        let mut remaining = size;
        let mut buf = [0u8; 8192];
        let mut ctx = digest::Context::new(&digest::SHA256);

        while remaining > 0 {
            let read_len = std::cmp::min(remaining, buf.len() as u64) as usize;
            let n = reader.read(&mut buf[..read_len])?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "truncated file data in archive during verify",
                ));
            }
            ctx.update(&buf[..n]);
            remaining -= n as u64;
            pb.inc(n as u64);
        }

        checked += 1;

        let calc = ctx.finish();
        if calc.as_ref() == expected_hash {
            ok += 1;
        } else {
            mismatched += 1;
        }
    }

    pb.finish_with_message("verify complete");

    let manifest_count = manifest.files.len();
    if checked != manifest_count {
        eprintln!(
            "warning: manifest lists {} files but archive contains {} entries",
            manifest_count, checked
        );
    }

    print_section("summary");
    print_kv("checked", checked.to_string());
    print_kv("ok", ok.to_string());
    print_kv("mismatched", mismatched.to_string());

    Ok(())
}
