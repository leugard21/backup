use std::{env, path::PathBuf};

#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub threads: Option<usize>,
    pub verify: bool,
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
}

impl BackupConfig {
    pub fn from_env() -> Result<Self, String> {
        let mut args = env::args().skip(1);
        let first_source = args
            .next()
            .ok_or_else(|| "missing <source-dir> path".to_string())?;
        Self::from_args(first_source, args)
    }

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

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--threads" | "-j" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "missing value for --threads".to_string())?;
                    let parsed: usize = value
                        .parse()
                        .map_err(|_| "invalid value for --threads".to_string())?;
                    threads = Some(parsed);
                }
                "--verify" => {
                    verify = true;
                }
                "--include" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "missing value for --include".to_string())?;
                    includes.push(value);
                }
                "--exclude" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "missing value for --exclude".to_string())?;
                    excludes.push(value);
                }
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
        })
    }
}
