mod pool;

use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Context;
use redis::ClientTlsConfig;

pub type Pool = deadpool::managed::Pool<pool::Manager>;

pub use pool::Manager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RedisTlsConfig<'a> {
    pub cert: Option<&'a Path>,
    pub key: Option<&'a Path>,
    pub ca: Option<&'a Path>,
}

#[derive(PartialEq, Eq, Hash, Default)]
struct RedisConfigKey {
    url: String,
    cert: Option<PathBuf>,
    key: Option<PathBuf>,
    ca: Option<PathBuf>,
}

/// A deduplicating factory for redis connection pools.
///
/// If you ask it to create a pool with the same details twice it will return the same pool
#[derive(Default)]
pub struct RedisPoolFactory {
    pools: HashMap<RedisConfigKey, Pool>,
}

impl RedisPoolFactory {
    pub fn pool(&mut self, url: &str, tls_config: Option<RedisTlsConfig<'_>>) -> anyhow::Result<Pool> {
        let key = {
            let mut config_key = RedisConfigKey {
                url: url.to_string(),
                ..Default::default()
            };
            if let Some(RedisTlsConfig { cert, key, ca }) = tls_config {
                config_key = RedisConfigKey {
                    cert: cert.map(ToOwned::to_owned),
                    key: key.map(ToOwned::to_owned),
                    ca: ca.map(ToOwned::to_owned),
                    ..config_key
                };
            }
            config_key
        };

        match self.pools.entry(key) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            Entry::Vacant(entry) => {
                let pool = new_pool(url, tls_config)?;
                entry.insert(pool.clone());
                Ok(pool)
            }
        }
    }
}

fn new_pool(url: &str, tls_config: Option<RedisTlsConfig<'_>>) -> anyhow::Result<Pool> {
    let tls_config = match tls_config {
        Some(tls) => {
            let client_tls = match tls.cert.zip(tls.key) {
                Some((cert, key)) => {
                    let mut client_cert = Vec::new();

                    File::open(cert)
                        .and_then(|file| BufReader::new(file).read_to_end(&mut client_cert))
                        .context("loading the Redis client certificate")?;

                    let mut client_key = Vec::new();

                    File::open(key)
                        .and_then(|file| BufReader::new(file).read_to_end(&mut client_key))
                        .context("loading the Redis client key")?;

                    Some(ClientTlsConfig {
                        client_cert,
                        client_key,
                    })
                }
                None => None,
            };

            let root_cert = match tls.ca {
                Some(path) => {
                    let mut ca = Vec::new();

                    File::open(path)
                        .and_then(|file| BufReader::new(file).read_to_end(&mut ca))
                        .context("loading the Redis CA certificate")?;

                    Some(ca)
                }
                None => None,
            };

            Some(pool::TlsConfig { client_tls, root_cert })
        }
        None => None,
    };

    let manager = match pool::Manager::new(url, tls_config) {
        Ok(manager) => manager,
        Err(e) => {
            tracing::error!("error creating a Redis pool: {e}");
            return Err(e.into());
        }
    };

    match Pool::builder(manager)
        .wait_timeout(Some(Duration::from_secs(5)))
        .create_timeout(Some(Duration::from_secs(10)))
        .runtime(deadpool::Runtime::Tokio1)
        .build()
    {
        Ok(pool) => Ok(pool),
        Err(e) => {
            tracing::error!("error creating a Redis pool: {e}");
            Err(e.into())
        }
    }
}
