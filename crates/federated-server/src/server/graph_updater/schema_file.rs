use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::server::{engine_reloader::GraphSender, gateway::GraphDefinition};
use either::Either;

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
            .unwrap_or_else(|_| Either::Left(SystemTime::now()))
            .map_right(|(hash, _)| hash);

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

            let (fingerprint, schema) = match fingerprint {
                Either::Left(modified) => (Either::Left(modified), None),
                Either::Right((hash, schema)) => (Either::Right(hash), Some(schema)),
            };

            if fingerprint != self.schema_fingerprint {
                self.schema_fingerprint = fingerprint;

                let schema = match schema {
                    Some(schema) => schema,
                    None => {
                        let Ok(schema) = tokio::fs::read_to_string(&self.schema_path).await else {
                            tracing::warn!(
                               "Could not load schema file in {}. The gateway will not use schema hot reload until file is available.",
                               self.schema_path.to_str().unwrap_or_default()
                           );

                            continue;
                        };

                        schema
                    }
                };

                tracing::info!("Detected a schema file update");

                self.sender
                    .send(GraphDefinition::Sdl(schema))
                    .await
                    .expect("channel must be up");
            } else {
                tracing::debug!("No schema file update detected");
            }
        }
    }
}

async fn schema_fingerprint(schema_path: &Path) -> Result<Either<SystemTime, (u64, String)>, ()> {
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
            tracing::debug!("The file system with the schema file does not support modification date. The schema hot reload will use a bit more CPU by hashing the file contents.");

            match tokio::fs::read_to_string(schema_path).await {
                Ok(schema) => {
                    // For huge schema files in this scenario, we do _not_ want to block the runtime
                    // for hashing.
                    let result = tokio::task::spawn_blocking(move || {
                        let mut hasher = DefaultHasher::new();
                        schema.hash(&mut hasher);
                        (hasher.finish(), schema)
                    })
                    .await;

                    match result {
                        Ok(result) => Ok(Either::Right(result)),
                        Err(e) => {
                            tracing::warn!("Internal error when hashing the schema changes: {e:?}",);
                            Err(())
                        }
                    }
                }
                Err(_) => {
                    tracing::warn!(
                        "Could not load schema metadata from the filesystem. The gateway will not use schema hot reload until file is readable in {}.",
                        schema_path.to_str().unwrap_or_default()
                    );

                    Err(())
                }
            }
        }
    }
}
