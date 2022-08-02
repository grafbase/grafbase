#![allow(dead_code)]

use crate::errors::ServerError;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

const FILE_WATCHER_INTERVAL: Duration = Duration::from_secs(1);

/// watches a file for write events, running a callback on each write
pub async fn start_watcher<P, T, R>(file: P, on_write: T) -> Result<(), ServerError>
where
    P: AsRef<Path> + Send + 'static,
    T: Fn() -> R + Send + 'static,
{
    let (notify_sender, notify_receiver) = mpsc::channel();

    let mut watcher: RecommendedWatcher = Watcher::new(notify_sender, FILE_WATCHER_INTERVAL)?;
    watcher.watch(&file, RecursiveMode::NonRecursive)?;

    tokio::task::spawn_blocking(move || -> Result<(), ServerError> {
        loop {
            match notify_receiver.recv() {
                Ok(DebouncedEvent::Write(_) | DebouncedEvent::Create(_)) => {
                    on_write();
                }
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
