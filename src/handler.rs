use std::{
    fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crossbeam_channel::Sender;
use notify::{
    Event, EventKind, RecursiveMode,
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
};
use serde::Serialize;

use crate::{
    filter::FsMessageFilter,
    metadata::{FsMetadata, FsMetadataStore},
};

pub struct FsEventHandler {
    sender: Sender<FsMessage>,
    filter: FsMessageFilter,
    store: FsMetadataStore,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FsMessageEvent {
    pub kind: FsMessageEventKind,
    pub path: PathBuf,
    pub metadata: Option<FsMetadata>,
}

#[derive(Serialize, Debug)]
pub enum FsMessageEventKind {
    Created,
    Modified,
    Removed,
}

impl FsMessageEvent {
    pub fn new(kind: FsMessageEventKind, path: PathBuf, metadata: Option<FsMetadata>) -> Self {
        Self {
            kind,
            path,
            metadata,
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FsMessageError {
    pub message: String,
}

impl FsMessageError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl From<notify::Error> for FsMessageError {
    fn from(value: notify::Error) -> Self {
        Self::new(value.to_string())
    }
}

impl From<io::Error> for FsMessageError {
    fn from(value: io::Error) -> Self {
        Self::new(value.to_string())
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum FsMessage {
    Event(FsMessageEvent),
    Error(FsMessageError),
}

impl From<FsMessageEvent> for FsMessage {
    fn from(value: FsMessageEvent) -> Self {
        Self::Event(value)
    }
}

impl From<FsMessageError> for FsMessage {
    fn from(value: FsMessageError) -> Self {
        Self::Error(value)
    }
}

impl From<notify::Error> for FsMessage {
    fn from(value: notify::Error) -> Self {
        Self::Error(value.into())
    }
}

impl From<io::Error> for FsMessage {
    fn from(value: io::Error) -> Self {
        Self::Error(value.into())
    }
}

impl FsEventHandler {
    pub fn new(sender: Sender<FsMessage>, filter: FsMessageFilter) -> Self {
        Self {
            sender,
            filter,
            store: FsMetadataStore::new(),
        }
    }

    pub fn init(&mut self, path: &Path, recursive_mode: RecursiveMode) -> Result<(), io::Error> {
        self.scan_dir(path, recursive_mode)
    }

    pub fn handle(&mut self, result: Result<Event, notify::Error>) {
        match result {
            Ok(event) => match event.kind {
                EventKind::Create(CreateKind::Any) => self.create_entry(&event.paths),
                EventKind::Modify(kind) => match kind {
                    ModifyKind::Any => self.modify_entry(&event.paths),
                    ModifyKind::Name(rename_mode) => match rename_mode {
                        RenameMode::From => self.remove_entry(&event.paths),
                        RenameMode::To => self.create_entry(&event.paths),
                        _ => {}
                    },
                    _ => {}
                },
                EventKind::Remove(RemoveKind::Any) => self.remove_entry(&event.paths),
                _ => {}
            },
            Err(error) => {
                let _ = self.sender.send(error.into());
            }
        }
    }

    fn create_entry(&mut self, paths: &[PathBuf]) {
        for path in paths {
            if self.filter.is_match(path) {
                let _ = match self.store.add(path) {
                    Ok(metadata) => self.sender.send(
                        FsMessageEvent::new(
                            FsMessageEventKind::Created,
                            path.clone(),
                            Some(metadata.clone()),
                        )
                        .into(),
                    ),
                    Err(error) => self.sender.send(error.into()),
                };
            }
        }
    }

    fn remove_entry(&mut self, paths: &[PathBuf]) {
        for path in paths {
            if self.filter.is_match(path) {
                let child_paths = self.store.child_paths(path);

                if !child_paths.is_empty() {
                    self.remove_entry(&child_paths);
                }

                let _ = self.sender.send(
                    FsMessageEvent::new(
                        FsMessageEventKind::Removed,
                        path.clone(),
                        self.store.remove(path),
                    )
                    .into(),
                );
            }
        }
    }

    fn modify_entry(&mut self, paths: &[PathBuf]) {
        for path in paths {
            if self.filter.is_match(path) {
                let old_ts = self
                    .store
                    .get(path)
                    .map_or(SystemTime::UNIX_EPOCH.into(), |metadata| metadata.modified);

                match self.store.add(path) {
                    Ok(metadata) => {
                        if metadata.modified > old_ts {
                            let _ = self.sender.send(
                                FsMessageEvent::new(
                                    FsMessageEventKind::Modified,
                                    path.clone(),
                                    Some(metadata.clone()),
                                )
                                .into(),
                            );
                        }
                    }
                    Err(error) => {
                        let _ = self.sender.send(error.into());
                    }
                }
            }
        }
    }

    fn scan_dir(&mut self, path: &Path, recursive_mode: RecursiveMode) -> Result<(), io::Error> {
        let entries = fs::read_dir(path)?;

        for entry in entries.flatten() {
            let path = entry.path();

            if self.filter.is_match(&path) {
                let metadata = self.store.add(&path)?;

                let _ = self.sender.send(
                    FsMessageEvent::new(
                        FsMessageEventKind::Created,
                        path.clone(),
                        Some(metadata.clone()),
                    )
                    .into(),
                );

                if let RecursiveMode::Recursive = recursive_mode {
                    if metadata.is_dir {
                        self.scan_dir(&path, recursive_mode)?;
                    }
                }
            }
        }

        Ok(())
    }
}
