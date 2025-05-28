use std::{fs, io::BufReader, sync::Arc, time::Duration};

pub use super::grafbase::sdk::kafka_client::*;

use crate::{resources::ProducerKind, state::WasiState};
use dashmap::Entry;
use rskafka::{
    BackoffConfig,
    client::{
        ClientBuilder, Credentials, SaslConfig,
        partition::{self, UnknownTopicHandling},
        producer::{BatchProducerBuilder, aggregator::RecordAggregator},
    },
};
use wasmtime::component::Resource;

impl Host for WasiState {}

impl HostKafkaProducer for WasiState {
    async fn connect(
        &mut self,
        name: String,
        servers: Vec<String>,
        topic: String,
        config: Option<KafkaProducerConfig>,
    ) -> wasmtime::Result<Result<Resource<KafkaProducer>, String>> {
        if !self.network_enabled() {
            return Ok(Err("Network operations are disabled".to_string()));
        }

        let producer = match self.kafka_producers().entry(name) {
            Entry::Occupied(occupied_entry) => occupied_entry.get().clone(),
            Entry::Vacant(vacant_entry) => {
                let producer = match create_producer(servers, topic, config).await {
                    Ok(value) => value,
                    Err(value) => return Ok(Err(value)),
                };

                vacant_entry.insert(producer.clone());

                producer
            }
        };

        Ok(Ok(self.push_resource(producer)?))
    }

    async fn produce(
        &mut self,
        self_: Resource<KafkaProducer>,
        key: Option<String>,
        value: Vec<u8>,
    ) -> wasmtime::Result<Result<(), String>> {
        let this = self.get(&self_)?;

        match this.produce(key, value).await {
            Ok(()) => Ok(Ok(())),
            Err(e) => Ok(Err(e)),
        }
    }

    async fn drop(&mut self, rep: Resource<KafkaProducer>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;

        Ok(())
    }
}

async fn create_producer(
    servers: Vec<String>,
    topic: String,
    config: Option<KafkaProducerConfig>,
) -> Result<KafkaProducer, String> {
    let mut builder = ClientBuilder::new(servers).backoff_config(BackoffConfig {
        init_backoff: Duration::from_millis(100),
        max_backoff: Duration::from_secs(1),
        base: 2.0,
        deadline: Some(Duration::from_secs(30)),
    });

    if let Some(KafkaProducerConfig {
        tls, authentication, ..
    }) = config.clone()
    {
        let tls_builder = match tls {
            None => None,
            Some(KafkaTlsConfig::SystemCa) => {
                let mut root_store = rustls::RootCertStore::empty();
                root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
                Some(rustls::ClientConfig::builder().with_root_certificates(root_store))
            }
            Some(KafkaTlsConfig::CustomCa(path)) => {
                let ca_cert_pem = match fs::File::open(&path) {
                    Ok(file) => file,
                    Err(err) => {
                        return Err(format!("error opening custom CA certificate file in `{path}`: {err}"));
                    }
                };

                let mut root_store = rustls::RootCertStore::empty();

                for cert in rustls_pemfile::certs(&mut BufReader::new(ca_cert_pem)) {
                    match cert {
                        Ok(cert) => root_store.add(cert).unwrap(),
                        Err(err) => {
                            return Err(format!("error loading certificate in `{path}`: {err}"));
                        }
                    }
                }

                Some(rustls::ClientConfig::builder().with_root_certificates(root_store))
            }
        };

        let tls_config = match authentication {
            None => tls_builder.map(|config| config.with_no_client_auth()),
            Some(KafkaAuthentication::SaslPlain(KafkaSaslPlainAuth { username, password })) => {
                builder = builder.sasl_config(SaslConfig::Plain(Credentials::new(username, password)));
                tls_builder.map(|config| config.with_no_client_auth())
            }
            Some(KafkaAuthentication::SaslScram(KafkaSaslScramAuth {
                username,
                password,
                mechanism: KafkaScramMechanism::Sha256,
            })) => {
                builder = builder.sasl_config(SaslConfig::ScramSha256(Credentials::new(username, password)));
                tls_builder.map(|config| config.with_no_client_auth())
            }
            Some(KafkaAuthentication::SaslScram(KafkaSaslScramAuth {
                username,
                password,
                mechanism: KafkaScramMechanism::Sha512,
            })) => {
                builder = builder.sasl_config(SaslConfig::ScramSha512(Credentials::new(username, password)));
                tls_builder.map(|config| config.with_no_client_auth())
            }
            Some(KafkaAuthentication::Mtls(KafkaMtlsAuth {
                client_cert_path,
                client_key_path,
            })) => {
                let client_cert_pem = match fs::File::open(&client_cert_path) {
                    Ok(pem) => pem,
                    Err(err) => return Err(format!("error opening client certificate file: {err}")),
                };

                let client_key_pem = match fs::File::open(&client_key_path) {
                    Ok(pem) => pem,
                    Err(err) => {
                        return Err(format!("error opening client key file in `{client_key_path}`: {err}"));
                    }
                };

                let client_cert_der =
                    match rustls_pemfile::certs(&mut BufReader::new(client_cert_pem)).collect::<Result<Vec<_>, _>>() {
                        Ok(der) => der,
                        Err(err) => {
                            return Err(format!(
                                "error reading client certificate file in `{client_cert_path}`: {err}"
                            ));
                        }
                    };

                let client_key_der = match rustls_pemfile::private_key(&mut BufReader::new(client_key_pem)) {
                    Ok(Some(der)) => der,
                    Ok(None) => return Err(format!("client key file in `{client_key_path}` is empty")),
                    Err(err) => {
                        return Err(format!("error opening client key file in `{client_key_path}`: {err}"));
                    }
                };

                let tls_builder = match tls_builder {
                    Some(builder) => builder,
                    None => {
                        let mut root_store = rustls::RootCertStore::empty();
                        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
                        rustls::ClientConfig::builder().with_root_certificates(root_store)
                    }
                };

                let config = match tls_builder.with_client_auth_cert(client_cert_der, client_key_der) {
                    Ok(config) => config,
                    Err(err) => return Err(format!("error configuring TLS client auth: {err}")),
                };

                Some(config)
            }
        };

        if let Some(tls_config) = tls_config {
            builder = builder.tls_config(Arc::new(tls_config));
        }
    }

    let client = match builder.build().await {
        Ok(client) => client,
        Err(err) => return Err(format!("error building Kafka client: {err}")),
    };

    let partitions = match config.as_ref().and_then(|c| c.partitions.clone()) {
        Some(partitions) => partitions,
        None => match client.list_topics().await {
            Ok(topics) => match topics.iter().find(|t| t.name == topic) {
                Some(topic) => topic.partitions.iter().copied().collect(),
                None => return Err(format!("topic not found: {topic}")),
            },
            Err(err) => return Err(format!("error listing topics: {err}")),
        },
    };

    let mut partition_clients = Vec::new();

    for partition in partitions {
        let partition_client = match client
            .partition_client(&topic, partition, UnknownTopicHandling::Error)
            .await
        {
            Ok(client) => client,
            Err(_) => todo!(),
        };

        let compression = match config.as_ref().map(|c| c.compression) {
            Some(KafkaProducerCompression::Gzip) => partition::Compression::Gzip,
            Some(KafkaProducerCompression::Lz4) => partition::Compression::Lz4,
            Some(KafkaProducerCompression::Snappy) => partition::Compression::Snappy,
            Some(KafkaProducerCompression::Zstd) => partition::Compression::Zstd,
            Some(KafkaProducerCompression::None) | None => partition::Compression::NoCompression,
        };

        let client = match config.as_ref().and_then(|c| c.batching.as_ref()) {
            Some(KafkaBatchConfig {
                linger_ms,
                batch_size_bytes,
            }) => {
                let builder = BatchProducerBuilder::new(Arc::new(partition_client))
                    .with_linger(Duration::from_millis(*linger_ms))
                    .with_compression(compression);

                let aggregator = RecordAggregator::new(*batch_size_bytes as usize);
                let batch_processor = builder.build(aggregator);

                ProducerKind::Batch(batch_processor)
            }
            None => ProducerKind::Single(partition_client, compression),
        };

        partition_clients.push(client);
    }

    Ok(KafkaProducer::new(partition_clients))
}
