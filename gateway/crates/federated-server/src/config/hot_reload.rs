use std::{collections::HashMap, fs, path::PathBuf, sync::OnceLock, time::Duration};

use grafbase_telemetry::span::GRAFBASE_TARGET;
use notify::{EventHandler, EventKind, PollWatcher, Watcher};
use runtime::rate_limiting::{GraphRateLimit, RateLimitKey};
use tokio::sync::watch;

use crate::Config;

type RateLimitData = HashMap<RateLimitKey<'static>, GraphRateLimit>;

pub(crate) struct ConfigWatcher {
    config_path: PathBuf,
    rate_limit_sender: watch::Sender<RateLimitData>,
}

impl ConfigWatcher {
    pub fn new(config_path: PathBuf, rate_limit_sender: watch::Sender<RateLimitData>) -> Self {
        Self {
            config_path,
            rate_limit_sender,
        }
    }

    pub fn watch(self) -> crate::Result<()> {
        static WATCHER: OnceLock<PollWatcher> = OnceLock::new();

        WATCHER.get_or_init(|| {
            let config = notify::Config::default().with_poll_interval(Duration::from_secs(1));
            let path = self.config_path.clone();
            let mut watcher = PollWatcher::new(self, config).expect("config watch init failed");

            watcher
                .watch(&path, notify::RecursiveMode::NonRecursive)
                .expect("config watch failed");

            watcher
        });

        Ok(())
    }

    fn reload_config(&self) -> crate::Result<()> {
        let config = match fs::read_to_string(&self.config_path) {
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

        let rate_limiting_configs = config
            .as_keyed_rate_limit_config()
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    runtime::rate_limiting::GraphRateLimit {
                        limit: v.limit,
                        duration: v.duration,
                    },
                )
            })
            .collect();

        self.rate_limit_sender.send(rate_limiting_configs)?;

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
