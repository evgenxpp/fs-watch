use std::{
    collections::{HashMap, hash_map::Entry},
    fs::Metadata,
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FsMetadata {
    pub is_dir: bool,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

impl FsMetadata {
    pub fn new(is_dir: bool, created: DateTime<Utc>, modified: DateTime<Utc>) -> Self {
        Self {
            is_dir,
            created,
            modified,
        }
    }
}

impl From<Metadata> for FsMetadata {
    fn from(value: Metadata) -> Self {
        let is_dir = value.is_dir();
        let created = value.created().unwrap_or(SystemTime::UNIX_EPOCH);
        let modified = value.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        FsMetadata::new(is_dir, created.into(), modified.into())
    }
}

#[derive(Debug)]
pub struct FsMetadataStore {
    data: HashMap<PathBuf, FsMetadata>,
}

impl FsMetadataStore {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get(&self, path: &Path) -> Option<&FsMetadata> {
        self.data.get(path)
    }

    pub fn add(&mut self, path: &Path) -> Result<&FsMetadata, io::Error> {
        let metadata: FsMetadata = path.metadata()?.into();

        Ok(match self.data.entry(path.into()) {
            Entry::Occupied(mut entry) => {
                entry.insert(metadata);
                entry.into_mut()
            }
            Entry::Vacant(entry) => entry.insert(metadata),
        })
    }

    pub fn remove(&mut self, path: &Path) -> Option<FsMetadata> {
        self.data.remove(path)
    }

    pub fn child_paths(&self, path: &Path) -> Vec<PathBuf> {
        if let Some(current) = self.get(path) {
            if current.is_dir {
                let prefix = path;

                return self
                    .data
                    .keys()
                    .filter(|key| key.starts_with(prefix) && key != &prefix)
                    .cloned()
                    .collect();
            }
        }

        Vec::new()
    }
}
