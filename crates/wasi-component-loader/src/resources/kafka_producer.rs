mod murmur2;

use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use rskafka::{
    chrono,
    client::{
        partition::{Compression, PartitionClient},
        producer::{BatchProducer, aggregator::RecordAggregator},
    },
    record::Record,
};

#[derive(Debug)]
pub enum ProducerKind {
    Batch(BatchProducer<RecordAggregator>),
    Single(PartitionClient, Compression),
}

impl ProducerKind {
    async fn produce(&self, record: Record) -> Result<(), String> {
        match self {
            ProducerKind::Batch(batch_producer) => {
                batch_producer
                    .produce(record)
                    .await
                    .map_err(|e| format!("Failed to produce message: {e}"))?;
            }
            ProducerKind::Single(partition_client, compression) => {
                partition_client
                    .produce(vec![record], *compression)
                    .await
                    .map_err(|e| format!("Failed to produce message: {e}"))?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct InnerProducer {
    partition_clients: Vec<ProducerKind>,
    round_robin_counter: AtomicU32,
}

impl InnerProducer {
    pub async fn produce(&self, key: Option<String>, value: Vec<u8>) -> Result<(), String> {
        let partition = match key.as_deref() {
            Some(key) if !key.is_empty() => {
                // The hasher returns a u32, compared to the Java version which returns i32.
                // The Java version can return negative values, so they use the mask to handle
                // that case.
                //
                // In our case, we convert the u32 to a positive i32 by using the mask. Simpler
                // would be to modulo the u32 by the length of the partition_clients vector,
                // but that would differ from the Java version and we want to partition exactly
                // like they do.
                let hash = murmur2::hash(key.as_bytes());
                (hash & 0x7fffffff) as i32 % self.partition_clients.len() as i32
            }
            _ => {
                let current = self.round_robin_counter.fetch_add(1, Ordering::Relaxed);
                (current % self.partition_clients.len() as u32) as i32
            }
        };

        let record = Record {
            key: key.map(|k| k.into_bytes()),
            value: Some(value),
            headers: BTreeMap::new(),
            timestamp: chrono::Utc::now(),
        };

        self.partition_clients[partition as usize]
            .produce(record)
            .await
            .map_err(|e| format!("Failed to produce message: {e}"))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct KafkaProducer {
    inner: Arc<InnerProducer>,
}

impl KafkaProducer {
    pub fn new(partition_clients: Vec<ProducerKind>) -> Self {
        Self {
            inner: Arc::new(InnerProducer {
                partition_clients,
                round_robin_counter: AtomicU32::new(0),
            }),
        }
    }

    pub async fn produce(&self, key: Option<String>, value: Vec<u8>) -> Result<(), String> {
        self.inner.produce(key, value).await
    }
}
