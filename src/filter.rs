use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

pub struct PathFilter {
    include: Option<GlobSet>,
    exclude: Option<GlobSet>,
}

impl PathFilter {
    pub fn from_patterns(includes: &[String], excludes: &[String]) -> Result<Self, String> {
        let include = if includes.is_empty() {
            None
        } else {
            let mut builder = GlobSetBuilder::new();
            for pat in includes {
                let glob =
                    Glob::new(pat).map_err(|e| format!("invalid include pattern {pat:?}: {e}"))?;
                builder.add(glob);
            }
            Some(
                builder
                    .build()
                    .map_err(|e| format!("failed to build include glob set: {e}"))?,
            )
        };

        let exclude = if excludes.is_empty() {
            None
        } else {
            let mut builder = GlobSetBuilder::new();
            for pat in excludes {
                let glob =
                    Glob::new(pat).map_err(|e| format!("invalid exclude pattern {pat:?}: {e}"))?;
                builder.add(glob);
            }
            Some(
                builder
                    .build()
                    .map_err(|e| format!("failed to build exclude glob set: {e}"))?,
            )
        };

        Ok(Self { include, exclude })
    }

    pub fn allow(&self, rel: &Path) -> bool {
        let s = rel.to_string_lossy().replace('\\', "/");

        if let Some(ex) = &self.exclude {
            if ex.is_match(&s) {
                return false;
            }
        }

        if let Some(inc) = &self.include {
            inc.is_match(&s)
        } else {
            true
        }
    }
}
