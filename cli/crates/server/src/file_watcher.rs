#![allow(dead_code)]

use crate::errors::ServerError;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

const FILE_WATCHER_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileEventType {
    Created,
    Removed,
    Changed,
}

/// watches a file for write events, running a callback on each write
pub async fn start_watcher<P, T>(file: P, on_write: T) -> Result<(), ServerError>
where
    P: AsRef<Path> + Send + 'static,
    T: Fn(PathBuf, FileEventType) + Send + 'static,
{
    let (notify_sender, notify_receiver) = mpsc::channel();

    let mut watcher: RecommendedWatcher = Watcher::new(notify_sender, FILE_WATCHER_INTERVAL)?;
    watcher.watch(&file, RecursiveMode::Recursive)?;

    tokio::task::spawn_blocking(move || -> Result<(), ServerError> {
        loop {
            match notify_receiver.recv() {
                Ok(DebouncedEvent::Create(path)) => on_write(path, FileEventType::Created),
                Ok(DebouncedEvent::Write(path)) => on_write(path, FileEventType::Changed),
                Ok(DebouncedEvent::Remove(path)) => on_write(path, FileEventType::Removed),
                // since `watcher` will go out of scope once the runtime restarts, we'll get a `RecvError`
                // here on reload, which allows us to stop the loop
                Err(_) => break,
                _ => {}
            }
        }
        Ok(())
    })
    .await?
}
