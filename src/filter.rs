use globset::{Glob, GlobSet};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct FsMessageFilter {
    mode: FsMessageFilterMode,
    globs: GlobSet,
}

#[derive(Clone, Debug)]
pub enum FsMessageFilterMode {
    OptIn,
    OptOut,
}

impl FsMessageFilter {
    pub fn create(mode: FsMessageFilterMode, globs: Vec<String>) -> Result<Self, globset::Error> {
        let mut builder = GlobSet::builder();

        for glob in globs {
            builder.add(Glob::new(&glob)?);
        }

        Ok(Self {
            mode,
            globs: builder.build()?,
        })
    }

    pub fn empty() -> Self {
        Self {
            mode: FsMessageFilterMode::OptOut,
            globs: GlobSet::empty(),
        }
    }

    pub fn is_match(&self, path: &Path) -> bool {
        match self.mode {
            FsMessageFilterMode::OptIn => self.globs.is_match(path),
            FsMessageFilterMode::OptOut => !self.globs.is_match(path),
        }
    }
}

impl Default for FsMessageFilter {
    fn default() -> Self {
        Self::empty()
    }
}
