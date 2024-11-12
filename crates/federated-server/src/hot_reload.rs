use std::{fs, path::PathBuf, sync::OnceLock, time::Duration};

use gateway_config::Config;
use notify::{EventHandler, EventKind, PollWatcher, Watcher};
use tokio::sync::watch;

/// A watcher for configuration files that monitors changes and sends updates.
///
/// The `ConfigWatcher` struct holds the path to the configuration file and a sender
/// channel that notifies subscribers of changes to the configuration.
pub(crate) struct ConfigWatcher {
    /// The path to the configuration file being watched.
    pub(crate) path: PathBuf,
    /// The sender for the watch channel used to notify about configuration updates.
    pub(crate) sender: watch::Sender<Config>,
}

impl ConfigWatcher {
    /// Initializes the `ConfigWatcher` with the given configuration and optional path for hot reloading.
    ///
    /// This function creates a new `ConfigWatcher` instance that monitors the specified configuration file.
    /// If a `hot_reload_config_path` is provided, it starts watching that file for changes.
    ///
    /// # Arguments
    ///
    /// - `config`: The initial configuration to be used.
    /// - `hot_reload_config_path`: An optional path to a configuration file that should be watched for changes.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `watch::Receiver<Config>` for receiving configuration updates,
    /// or an error if initialization fails.
    pub fn init(config: Config, hot_reload_config_path: Option<PathBuf>) -> crate::Result<watch::Receiver<Config>> {
        let (sender, receiver) = watch::channel(config);

        if let Some(path) = hot_reload_config_path {
            Self { path, sender }.start()?
        }

        Ok(receiver)
    }

    /// Starts the configuration watcher, initializing the file watching process.
    ///
    /// This function sets up the watcher to monitor the specified configuration file for changes.
    /// It uses a polling mechanism to check for modifications and triggers the appropriate events.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the watcher initialization.
    fn start(self) -> crate::Result<()> {
        static WATCHER: OnceLock<PollWatcher> = OnceLock::new();

        WATCHER.get_or_init(|| {
            let config = notify::Config::default().with_poll_interval(Duration::from_secs(1));
            let path = self.path.clone();
            let mut watcher = PollWatcher::new(self, config).expect("config watch init failed");

            watcher
                .watch(&path, notify::RecursiveMode::NonRecursive)
                .expect("config watch failed");

            watcher
        });

        Ok(())
    }

    /// Reloads the configuration from the specified file path.
    ///
    /// This function reads the configuration file, parses its contents, and sends the new configuration
    /// to the subscribers through the sender channel. If an error occurs while reading or parsing the
    /// configuration file, it logs the error but does not propagate it upwards.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the reload operation. If an error occurs, it is logged
    /// but the function returns `Ok(())` to indicate that the operation completed without crashing.
    fn reload_config(&self) -> crate::Result<()> {
        let config = match fs::read_to_string(&self.path) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("error reading gateway config: {e}");

                return Ok(());
            }
        };

        let config: Config = match toml::from_str(&config) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("error parsing gateway config: {e}");

                return Ok(());
            }
        };

        self.sender.send(config)?;

        Ok(())
    }
}

impl EventHandler for ConfigWatcher {
    fn handle_event(&mut self, event: notify::Result<notify::Event>) {
        match event.map(|e| e.kind) {
            Ok(EventKind::Any | EventKind::Create(_) | EventKind::Modify(_) | EventKind::Other) => {
                tracing::debug!("reloading configuration file");

                if let Err(e) = self.reload_config() {
                    tracing::error!("error reloading gateway config: {e}");
                };
            }
            Ok(_) => (),
            Err(e) => {
                tracing::error!("error reading gateway config: {e}");
            }
        }
    }
}
