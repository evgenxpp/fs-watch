use std::{
    io,
    path::Path,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crossbeam_channel::Sender;
use notify::{
    Config, Event, ReadDirectoryChangesWatcher, RecommendedWatcher, RecursiveMode,
    Watcher as NotifyWatcher,
};

use crate::{
    filter::FsMessageFilter,
    handler::{FsEventHandler, FsMessage},
};

pub struct Watcher {
    watcher: ReadDirectoryChangesWatcher,
    handler: Arc<Mutex<FsEventHandler>>,
}

impl Watcher {
    pub fn create(
        sender: Sender<FsMessage>,
        filter: Option<FsMessageFilter>,
    ) -> Result<Self, notify::Error> {
        let filter = filter.unwrap_or_default();
        let handler = Arc::new(Mutex::new(FsEventHandler::new(sender, filter)));
        let watcher = RecommendedWatcher::new(
            {
                let handler = Arc::clone(&handler);

                move |result: Result<Event, notify::Error>| {
                    let mut handler = handler.lock().unwrap();

                    handler.handle(result)
                }
            },
            Config::default()
                .with_compare_contents(false)
                .with_follow_symlinks(false),
        )?;

        Ok(Self { watcher, handler })
    }

    pub fn watch(
        &mut self,
        path: &Path,
        recursive_mode: RecursiveMode,
    ) -> Result<JoinHandle<Result<(), io::Error>>, notify::Error> {
        self.watcher.watch(path, recursive_mode)?;

        let join_handle = thread::spawn({
            let path = path.to_path_buf();
            let handler = Arc::clone(&self.handler);

            move || {
                let mut handler = handler.lock().unwrap();
                handler.init(&path, recursive_mode)
            }
        });

        Ok(join_handle)
    }
}
