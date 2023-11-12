use crate::consts::DOT_ENV_FILE_NAME;
use crate::errors::ServerError;
use crate::udf_builder::LOCK_FILE_NAMES;
use common::consts::{DOT_GRAFBASE_DIRECTORY_NAME, GRAFBASE_SCHEMA_FILE_NAME};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::runtime::Handle;

const FILE_WATCHER_INTERVAL: Duration = Duration::from_secs(1);

/// watches a path for file system events, running a callback on each event
pub async fn start_watcher<P, T>(path: P, on_change: T) -> Result<(), ServerError>
where
    P: AsRef<Path> + Send + 'static,
    T: Fn(&PathBuf) + Send + 'static,
{
    let (notify_sender, mut notify_receiver) = tokio::sync::mpsc::channel(1);

    let handle = Handle::current();

    let mut debouncer = new_debouncer(FILE_WATCHER_INTERVAL, None, move |res| {
        handle.block_on(async { notify_sender.send(res).await.expect("must be open") });
    })?;

    debouncer.watcher().watch(path.as_ref(), RecursiveMode::Recursive)?;

    loop {
        match notify_receiver.recv().await {
            Some(Ok(events)) => {
                // for the purposes of display, we need the last non ignored event
                if let Some(event) = events
                    .iter()
                    .rev()
                    .find(|event| should_handle_change(&event.path, path.as_ref()))
                {
                    on_change(&event.path);
                }
            }

            Some(Err(errors)) => {
                if let Some(error) = errors
                    .into_iter()
                    .find(|error| error.paths.contains(&path.as_ref().to_owned()))
                {
                    // an error with the root path, non recoverable
                    return Err(ServerError::FileWatcher(error));
                }
                // errors for specific files, ignored
            }
            // unreachable, should always be stopped externally by `select!`
            None => {}
        }
    }
}

const ROOT_FILE_WHITELIST: [&str; 2] = [GRAFBASE_SCHEMA_FILE_NAME, DOT_ENV_FILE_NAME];
const EXTENSION_WHITELIST: [&str; 11] = [
    "js", "ts", "jsx", "tsx", "mjs", "mts", ".wasm", "cjs", "json", "yaml", "yml",
];
const DIRECTORY_BLACKLIST: &[&str] = &[DOT_GRAFBASE_DIRECTORY_NAME, "node_modules", "generated"];

fn should_handle_change(path: &Path, root: &Path) -> bool {
    is_whitelisted_root_file(path, root)
        || (!(is_likely_a_directory(path) || in_blacklisted_directory(path, root) || is_lock_file_path(path, root))
            && has_whitelisted_extension(path))
}

fn is_lock_file_path(path: &Path, root: &Path) -> bool {
    LOCK_FILE_NAMES
        .iter()
        .any(|(file_name, _)| root.join(file_name) == path)
}

fn is_likely_a_directory(path: &Path) -> bool {
    // we can't know if something was a directory after removal, so this is based on best effort.
    // if a directory matching a name in `ROOT_FILE_WHITELIST` is removed, it'll trigger `on_change`, although that's an unlikely edge case.
    // note that we're not using `.is_file()` here since it'd have a false negative for removal.
    // also avoiding notifying on files that we can't access by using the metadata version of `is_dir`
    path.metadata().map(|metadata| metadata.is_dir()).ok().unwrap_or(true)
}

fn is_whitelisted_root_file(path: &Path, root: &Path) -> bool {
    ROOT_FILE_WHITELIST.iter().any(|file_name| root.join(file_name) == path)
}

fn in_blacklisted_directory(path: &Path, root: &Path) -> bool {
    // we only blacklist directories under the grafbase directory
    path.strip_prefix(root)
        .expect("must contain root directory")
        .iter()
        .filter_map(std::ffi::OsStr::to_str)
        .any(|path_part| DIRECTORY_BLACKLIST.contains(&path_part))
}

fn has_whitelisted_extension(path: &Path) -> bool {
    path.extension()
        .iter()
        .filter_map(|extension| extension.to_str())
        .any(|extension| EXTENSION_WHITELIST.contains(&extension))
}
