use std::{
    hash::{DefaultHasher, Hasher},
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::server::{engine_reloader::GraphSender, gateway::GraphDefinition};
use either::Either;
use futures_lite::StreamExt;
use tokio_util::codec::{BytesCodec, FramedRead};

/// How often we poll updates to the graph.
const TICK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

/// A struct representing a schema file graph updater, which is responsible for polling updates
/// from the file system to a schema file, and initializing a new instance of the GraphQL engine.
pub struct SchemaFileGraphUpdater {
    schema_path: PathBuf,
    schema_fingerprint: Either<SystemTime, u64>,
    sender: GraphSender,
}

impl SchemaFileGraphUpdater {
    /// Initialize a new graph updater.
    pub async fn new(schema_path: PathBuf, sender: GraphSender) -> Self {
        let schema_fingerprint = schema_fingerprint(&schema_path)
            .await
            .unwrap_or_else(|_| Either::Left(SystemTime::now()));

        Self {
            schema_path,
            schema_fingerprint,
            sender,
        }
    }

    /// Start polling changes to the file. We use a simple interval check to be able to run
    /// in any kind of environment. An fs event watcher would be faster, but might not be guaranteed
    /// to work in environments such as docker on macOS, with network file systems such as NFS
    /// or with mounted volumes in Kubernetes.
    pub async fn poll(mut self) {
        let mut interval = tokio::time::interval(TICK_INTERVAL);

        loop {
            interval.tick().await;

            let Ok(fingerprint) = schema_fingerprint(&self.schema_path).await else {
                continue;
            };

            if fingerprint != self.schema_fingerprint {
                self.schema_fingerprint = fingerprint;

                let Ok(schema) = tokio::fs::read_to_string(&self.schema_path).await else {
                    tracing::warn!(
                        "Could not load schema file in {}. The gateway will not use schema hot reload until file is available.",
                        self.schema_path.to_str().unwrap_or_default()
                    );

                    continue;
                };

                tracing::info!("Detected a schema file update");

                self.sender
                    .send(GraphDefinition::Sdl(schema))
                    .await
                    .expect("channel must be up");
            } else {
                tracing::trace!("No schema file update detected");
            }
        }
    }
}

async fn schema_fingerprint(schema_path: &Path) -> Result<Either<SystemTime, u64>, ()> {
    let metadata = match tokio::fs::metadata(schema_path).await {
        Ok(metadata) => metadata,
        Err(_) => {
            tracing::warn!(
                "Could not load schema metadata from the filesystem. The gateway will not use schema hot reload until file is readable in {}.",
                schema_path.to_str().unwrap_or_default(),
            );

            return Err(());
        }
    };

    match metadata.modified() {
        // If the file system supports modification date, we can use it to check for changes with less resources.
        Ok(modified) => Ok(Either::Left(modified)),
        // This is a fallback for file systems that do not support modification date. It loads and hashes the file
        // contents to check for changes.
        Err(_) => {
            tracing::debug!(
                "The file system with the schema file does not support modification date. The schema hot reload will use a bit more CPU by hashing the file contents."
            );

            let Ok(file) = tokio::fs::File::open(schema_path).await else {
                tracing::warn!(
                    "Could not open schema file in {}. The gateway will not use schema hot reload until file is available.",
                    schema_path.to_str().unwrap_or_default()
                );

                return Err(());
            };

            let mut stream = FramedRead::new(file, BytesCodec::new());
            let mut hasher = DefaultHasher::new();

            while let Some(chunk) = stream.next().await {
                tracing::warn!(
                    "Could not open schema file in {}. The gateway will not use schema hot reload until file is available.",
                    schema_path.to_str().unwrap_or_default()
                );

                let Ok(chunk) = chunk else {
                    return Err(());
                };

                hasher.write(&chunk);
            }

            Ok(Either::Right(hasher.finish()))
        }
    }
}
