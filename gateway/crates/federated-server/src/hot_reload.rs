use std::{fs, path::PathBuf, sync::OnceLock, time::Duration};

use gateway_config::Config;
use grafbase_telemetry::span::GRAFBASE_TARGET;
use notify::{EventHandler, EventKind, PollWatcher, Watcher};
use tokio::sync::watch;

pub(crate) struct ConfigWatcher {
    path: PathBuf,
    sender: watch::Sender<Config>,
}

impl ConfigWatcher {
    pub fn init(config: Config, hot_reload_config_path: Option<PathBuf>) -> crate::Result<watch::Receiver<Config>> {
        let (sender, receiver) = watch::channel(config);
        if let Some(path) = hot_reload_config_path {
            Self { path, sender }.start()?
        }
        Ok(receiver)
    }

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

    fn reload_config(&self) -> crate::Result<()> {
        let config = match fs::read_to_string(&self.path) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!(target: GRAFBASE_TARGET, "error reading gateway config: {e}");

                return Ok(());
            }
        };

        let config: Config = match toml::from_str(&config) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!(target: GRAFBASE_TARGET, "error parsing gateway config: {e}");

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
                tracing::debug!(target: GRAFBASE_TARGET, "reloading configuration file");

                if let Err(e) = self.reload_config() {
                    tracing::error!(target: GRAFBASE_TARGET, "error reloading gateway config: {e}");
                };
            }
            Ok(_) => (),
            Err(e) => {
                tracing::error!(target: GRAFBASE_TARGET, "error reading gateway config: {e}");
            }
        }
    }
}
