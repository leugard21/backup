use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub threads: Option<usize>,
    pub verify: bool,
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
    pub dry_run: bool,
}

impl BackupConfig {
    pub fn from_args<I>(first_source: String, mut args: I) -> Result<Self, String>
    where
        I: Iterator<Item = String>,
    {
        let destination = args
            .next()
            .ok_or_else(|| "missing <backup-dir> path".to_string())?;

        let mut threads = None;
        let mut verify = false;
        let mut includes = Vec::new();
        let mut excludes = Vec::new();
        let mut dry_run = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--threads" | "-j" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "missing value for --threads".to_string())?;
                    threads = Some(
                        value
                            .parse()
                            .map_err(|_| "invalid value for --threads".to_string())?,
                    );
                }
                "--verify" => verify = true,
                "--include" => {
                    let v = args
                        .next()
                        .ok_or_else(|| "missing value for --include".to_string())?;
                    includes.push(v);
                }
                "--exclude" => {
                    let v = args
                        .next()
                        .ok_or_else(|| "missing value for --exclude".to_string())?;
                    excludes.push(v);
                }
                "--dry-run" => dry_run = true,
                other => {
                    return Err(format!("unknown argument: {other}"));
                }
            }
        }

        Ok(Self {
            source: PathBuf::from(first_source),
            destination: PathBuf::from(destination),
            threads,
            verify,
            includes,
            excludes,
            dry_run,
        })
    }
}
