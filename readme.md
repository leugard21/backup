# Backup

A fast, parallel file backup utility written in Rust that creates single-file archives with SHA-256 integrity verification.

## Features

- **Parallel Processing**: Leverages Rayon for multi-threaded file hashing and backup operations
- **SHA-256 Hashing**: Every file is hashed for integrity verification during restore
- **Single-File Archives**: Creates `.backup` files containing all source files and metadata
- **Progress Tracking**: Visual progress bars for hashing and backup operations
- **Restore with Verification**: Automatically verifies file integrity when restoring from backups
- **Inspect Archives**: View backup metadata and file listings without extracting

## Installation

### Build from Source

```bash
cargo build --release
```

The binary will be available at `target/release/backup`.

## Usage

### Create a Backup

```bash
backup <source-dir> <backup-dir> [--threads N] [--verify]
```

**Arguments:**
- `<source-dir>`: Directory to back up
- `<backup-dir>`: Directory where the `.backup` file will be created
- `--threads N` or `-j N`: Number of threads to use for parallel processing (optional)
- `--verify`: Enable verification after backup (optional, not yet implemented)

**Example:**
```bash
backup /home/user/documents /mnt/backups --threads 8
```

This creates a timestamped backup file like `documents-1700000000.backup` in `/mnt/backups/`.

### Restore a Backup

```bash
backup restore <backup-file> <restore-dir>
```

**Arguments:**
- `<backup-file>`: Path to the `.backup` file
- `<restore-dir>`: Directory where files will be restored

**Example:**
```bash
backup restore /mnt/backups/documents-1700000000.backup /home/user/restored
```

Files are restored with automatic SHA-256 verification. Any hash mismatches are reported.

### Inspect a Backup

```bash
backup inspect <backup-file>
```

**Example:**
```bash
backup inspect /mnt/backups/documents-1700000000.backup
```

Displays backup metadata including file count, total size, and file listings.

## Backup File Format

The `.backup` file format is a custom binary format:

```
[Magic: "BKUP" (4 bytes)]
[Version: u32 (4 bytes)]
[Manifest Length: u64 (8 bytes)]
[Manifest JSON (variable length)]
[File Entries...]
```

Each file entry contains:
```
[Path Length: u16 (2 bytes)]
[Path: UTF-8 string (variable)]
[File Size: u64 (8 bytes)]
[SHA-256 Hash: 32 bytes]
[File Data: raw bytes]
```

## Dependencies

- **rayon**: Parallel processing
- **ring**: SHA-256 hashing
- **walkdir**: Directory traversal
- **indicatif**: Progress bars
- **serde** & **serde_json**: Manifest serialization

## Project Structure

```
src/
├── main.rs          # CLI entry point and command routing
├── config.rs        # Configuration parsing
├── fs_scan.rs       # Directory scanning
├── hasher.rs        # SHA-256 file hashing
├── pipeline.rs      # Parallel hashing pipeline
├── manifest.rs      # Backup manifest generation
├── backup_file.rs   # Archive creation
├── restore.rs       # Archive extraction and verification
├── inspect.rs       # Archive inspection
├── types.rs         # Common types
└── validation.rs    # Path validation
```

## Performance

The backup utility is designed for speed:
- Parallel file hashing using all available CPU cores
- Buffered I/O for efficient file reading/writing
- Minimal memory overhead with streaming operations

## Limitations

- Maximum path length: 65,535 bytes (u16::MAX)
- The `--verify` flag for post-backup verification is not yet implemented
- Archive format version is currently fixed at v1

## License

This project is open source. See the repository for license details.

## Contributing

Contributions are welcome! Please ensure code follows Rust best practices and includes appropriate error handling.
