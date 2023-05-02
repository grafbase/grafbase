use crate::consts::DOT_ENV_FILE;
use crate::errors::ServerError;
use common::consts::GRAFBASE_SCHEMA_FILE_NAME;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

const FILE_WATCHER_INTERVAL: Duration = Duration::from_secs(1);

/// watches a path for file system events, running a callback on each event
pub async fn start_watcher<P, T>(path: P, on_change: T) -> Result<(), ServerError>
where
    P: AsRef<Path> + Send + 'static,
    T: Fn(&PathBuf) + Send + 'static,
{
    let (notify_sender, notify_receiver) = mpsc::channel();

    let mut debouncer = new_debouncer(FILE_WATCHER_INTERVAL, None, notify_sender)?;

    debouncer.watcher().watch(path.as_ref(), RecursiveMode::Recursive)?;

    tokio::task::spawn_blocking(move || -> Result<(), ServerError> {
        loop {
            match notify_receiver.recv() {
                Ok(Ok(events)) => {
                    // for the purposes of display, we need the last non ignored event
                    if let Some(event) = events
                        .iter()
                        .rev()
                        .find(|event| non_ignored_path(&event.path, path.as_ref()))
                    {
                        on_change(&event.path);
                    }
                }
                // an error with the root path, non recoverable
                Ok(Err(errors))
                    if errors
                        .iter()
                        .any(|error| error.paths.contains(&path.as_ref().to_owned())) =>
                {
                    return Err(ServerError::FileWatcher(errors.into_iter().last().expect("must exist")))
                }
                // errors for specific files
                Ok(Err(_)) => {}
                // since `watcher` will go out of scope once the runtime restarts, we'll get a `RecvError`
                // here on reload, which allows us to stop the loop
                Err(_) => {
                    debouncer.watcher().unwatch(path.as_ref())?;
                    break;
                }
            }
        }
        Ok(())
    })
    .await?
}

const ROOT_FILE_WHITELIST: [&str; 2] = [GRAFBASE_SCHEMA_FILE_NAME, DOT_ENV_FILE];
const EXTENSION_WHITELIST: [&str; 5] = ["js", "ts", "json", "yaml", "yml"];
const DIRECTORY_BLACKLIST: [&str; 1] = ["node_modules"];

fn non_ignored_path(path: &Path, root: &Path) -> bool {
    likely_not_a_dir(path)
        && (whitelisted_root_file(path, root) || (!in_blacklisted_directory(path, root) && whitelisted_extension(path)))
}

fn likely_not_a_dir(path: &Path) -> bool {
    // we can't know if something was a directory after removal, so this is based on best effort.
    // if a directory matching a name in `ROOT_FILE_WHITELIST` is removed, it'll trigger `on_change`, although that's an unlikely edge case.
    // note that we're not using `.is_file()` here since it'd have a false negative for removal.
    // also avoiding notifying on files that we can't access by using the metadata version of `is_dir`
    path.metadata().map(|metadata| metadata.is_dir()).ok() == Some(false)
}

fn whitelisted_root_file(path: &Path, root: &Path) -> bool {
    let in_root = path.parent().filter(|parent| *parent == root).is_some();
    in_root
        && path
            .file_name()
            .and_then(OsStr::to_str)
            .filter(|file_name| ROOT_FILE_WHITELIST.contains(file_name))
            .is_some()
}

fn in_blacklisted_directory(path: &Path, root: &Path) -> bool {
    // we only blacklist directories under the grafbase directory
    path.strip_prefix(root)
        .expect("must contain root directory")
        .iter()
        .any(|path_part| {
            path_part
                .to_str()
                .filter(|path_part| DIRECTORY_BLACKLIST.contains(path_part))
                .is_some()
        })
}

fn whitelisted_extension(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .filter(|extension| EXTENSION_WHITELIST.contains(extension))
        .is_some()
}
