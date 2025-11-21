use std::{env, path::PathBuf};

#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub threads: Option<usize>,
    pub verify: bool,
}

impl BackupConfig {
    pub fn from_env() -> Result<Self, String> {
        let mut args = env::args().skip(1);

        let source = args
            .next()
            .ok_or_else(|| "missing <source> path".to_string())?;
        let destination: String = args
            .next()
            .ok_or_else(|| "missing <destination> path".to_string())?;

        let mut threads = None;
        let mut verify = false;

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
                other => {
                    return Err(format!("unknown argument: {other}"));
                }
            }
        }

        Ok(Self {
            source: PathBuf::from(source),
            destination: PathBuf::from(destination),
            threads,
            verify,
        })
    }
}
