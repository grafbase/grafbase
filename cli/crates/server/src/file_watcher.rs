use crate::consts::DOT_ENV_FILE_NAME;
use crate::errors::ServerError;
use crate::udf_builder::LOCK_FILE_NAMES;
use common::consts::{
    DOT_GRAFBASE_DIRECTORY_NAME, GRAFBASE_DIRECTORY_NAME, GRAFBASE_SCHEMA_FILE_NAME, GRAFBASE_TS_CONFIG_FILE_NAME,
};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::sync::broadcast::error::RecvError;
use tokio::task::JoinSet;

const FILE_WATCHER_INTERVAL: Duration = Duration::from_secs(1);

pub struct Watcher {
    join_set: tokio::task::JoinSet<Result<(), ServerError>>,
    receiver: tokio::sync::broadcast::Receiver<PathBuf>,
}

impl Watcher {
    pub async fn start<P>(path: P) -> Result<Watcher, ServerError>
    where
        P: AsRef<Path> + Send + 'static,
    {
        start_watcher(path).await
    }

    pub fn file_changes(&self) -> ChangeStream {
        ChangeStream {
            receiver: self.receiver.resubscribe(),
        }
    }

    pub async fn shutdown(mut self) -> Result<(), ServerError> {
        self.join_set.abort_all();
        match self.join_set.join_next().await {
            Some(Ok(result)) => result,
            Some(Err(join_error)) => {
                join_error.into_panic();
                Ok(())
            }
            None => unreachable!(),
        }
    }
}

pub struct ChangeStream {
    receiver: tokio::sync::broadcast::Receiver<PathBuf>,
}

async fn start_watcher<P>(path: P) -> Result<Watcher, ServerError>
where
    P: AsRef<Path> + Send + 'static,
{
    tracing::trace!("starting file watcher");

    let (notify_sender, mut notify_receiver) = tokio::sync::mpsc::channel(1);

    let handle = Handle::current();

    let mut debouncer = new_debouncer(FILE_WATCHER_INTERVAL, None, move |res| {
        handle.block_on(async { notify_sender.send(res).await.expect("must be open") });
    })?;

    debouncer.watcher().watch(path.as_ref(), RecursiveMode::Recursive)?;

    let (change_sender, change_receiver) = tokio::sync::broadcast::channel(128);

    let mut join_set = JoinSet::new();

    join_set.spawn(async move {
        // Move the debouncer into the task so it doesn't get dropped
        #[allow(unused)]
        let debouncer = debouncer;

        let root_path = path.as_ref();
        loop {
            match notify_receiver.recv().await {
                Some(Ok(events)) => {
                    // for the purposes of display, we need the last non ignored event
                    if let Some(event) = events
                        .into_iter()
                        .rev()
                        .find(|event| should_handle_change(&event.path, root_path))
                    {
                        let relative_path = event
                            .path
                            .strip_prefix(root_path)
                            .expect("must suceed by definition")
                            .to_owned();

                        if change_sender.send(relative_path).is_err() {
                            // Receiver has been dropped so we should shut down
                            return Ok(());
                        };
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
                None => return Ok(()),
            }
        }
    });

    Ok(Watcher {
        join_set,
        receiver: change_receiver,
    })
}

impl ChangeStream {
    pub async fn next(&mut self) -> Option<PathBuf> {
        loop {
            match self.receiver.recv().await {
                Ok(next) => return Some(next),
                Err(RecvError::Closed) => return None,
                Err(RecvError::Lagged(_)) => {
                    // Do we care?  I don't think we do
                }
            }
        }
    }
}

impl Clone for ChangeStream {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver.resubscribe(),
        }
    }
}

const ROOT_FILE_WHITELIST: &[&str] = &[
    GRAFBASE_SCHEMA_FILE_NAME,
    GRAFBASE_TS_CONFIG_FILE_NAME,
    DOT_ENV_FILE_NAME,
];
const EXTENSION_WHITELIST: &[&str] = &[
    "js", "ts", "jsx", "tsx", "mjs", "mts", "wasm", "cjs", "json", "yaml", "yml",
];
const DIRECTORY_BLACKLIST: &[&str] = &[DOT_GRAFBASE_DIRECTORY_NAME, "node_modules", "generated"];
const ROOT_WHITELIST: &[&str] = &[GRAFBASE_DIRECTORY_NAME, "resolvers", "auth"];

fn should_handle_change(path: &Path, root: &Path) -> bool {
    is_whitelisted_root_file(path, root)
        || in_whitelisted_root(path, root)
            && (!(is_likely_a_directory(path) || in_blacklisted_directory(path, root) || is_lock_file_path(path, root))
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
    path.metadata().map(|metadata| metadata.is_dir()).ok().unwrap_or(false)
}

fn is_whitelisted_root_file(path: &Path, root: &Path) -> bool {
    ROOT_FILE_WHITELIST
        .iter()
        .any(|file_name| (root.join(file_name) == path) || (root.join(GRAFBASE_DIRECTORY_NAME).join(file_name) == path))
}

fn in_blacklisted_directory(path: &Path, root: &Path) -> bool {
    // we only blacklist directories under the project directory
    path.strip_prefix(root)
        .expect("must contain root directory")
        .iter()
        .filter_map(std::ffi::OsStr::to_str)
        .any(|path_part| DIRECTORY_BLACKLIST.contains(&path_part))
}

fn in_whitelisted_root(path: &Path, root: &Path) -> bool {
    path.strip_prefix(root)
        .expect("must contain root directory")
        .iter()
        .next()
        .and_then(|root| root.to_str())
        .is_some_and(|root| ROOT_WHITELIST.contains(&root))
}

fn has_whitelisted_extension(path: &Path) -> bool {
    path.extension()
        .iter()
        .filter_map(|extension| extension.to_str())
        .any(|extension| EXTENSION_WHITELIST.contains(&extension))
}

#[test]
fn test_should_handle_change() {
    let root = Path::new("/Users/name/project");

    let handled_paths = &[
        "grafbase.config.ts",
        "schema.graphql",
        "grafbase/schema.graphql",
        "grafbase/grafbase.config.ts",
        "grafbase/file.yml",
        "resolvers/file.js",
        "auth/file.js",
        ".env",
    ];

    for path in handled_paths {
        let current = &root.join(path);
        let should_handle = should_handle_change(current, root);
        assert!(should_handle, "current path: {}", current.to_string_lossy());
    }

    let unhandled_paths = &[
        "file.txt",
        "grafbase/file.txt",
        "resolvers/file.txt",
        "file.ts",
        "resolvers/node_modules/file.ts",
        "target/file.yml",
        ".envrc",
        "resolvers/.env",
        "grafbase/blah/schema.graphql",
    ];

    for path in unhandled_paths {
        let current = &root.join(path);
        let should_handle = should_handle_change(current, root);
        assert!(!should_handle, "current path: {}", current.to_string_lossy());
    }
}
