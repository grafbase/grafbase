use std::ops::{Deref, DerefMut};

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde_json::Value;
use tokio_postgres::Transaction;

use crate::transport::ext::TransportTransactionExt;
use crate::{error::Error, transport::Transport};

use super::executor;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Transport for Transaction<'_> {
    async fn close(self) -> crate::Result<()> {
        Ok(())
    }

    async fn parameterized_execute(&self, query: &str, params: Vec<Value>) -> crate::Result<i64> {
        executor::execute(self, query, params).await
    }

    fn parameterized_query<'a>(&'a self, query: &'a str, params: Vec<Value>) -> BoxStream<'a, Result<Value, Error>> {
        executor::query(self, query, params)
    }

    // we don't care about this in a tx
    fn connection_string(&self) -> &str {
        ""
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl TransportTransactionExt for Transaction<'_> {
    async fn transaction(&mut self) -> crate::Result<TransportTransaction<'_>> {
        self.transaction()
            .await
            .map(TransportTransaction::Direct)
            .map_err(|e| Error::Transaction(e.to_string()))
    }
}

pub enum TransportTransaction<'a> {
    #[cfg(feature = "pooling")]
    Pooled(deadpool_postgres::Transaction<'a>),
    Direct(Transaction<'a>),
}

impl<'a> TransportTransaction<'a> {
    pub async fn commit(self) -> Result<(), Error> {
        match self {
            #[cfg(feature = "pooling")]
            TransportTransaction::Pooled(tx) => tx.commit().await.map_err(|e| Error::FailedCommit(e.to_string())),
            TransportTransaction::Direct(tx) => tx.commit().await.map_err(|e| Error::FailedCommit(e.to_string())),
        }
    }
}

impl<'a> Deref for TransportTransaction<'a> {
    type Target = Transaction<'a>;

    fn deref(&self) -> &Self::Target {
        match self {
            #[cfg(feature = "pooling")]
            TransportTransaction::Pooled(tx) => tx.deref(),
            TransportTransaction::Direct(tx) => tx,
        }
    }
}

impl<'a> DerefMut for TransportTransaction<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            #[cfg(feature = "pooling")]
            TransportTransaction::Pooled(tx) => tx.deref_mut(),
            TransportTransaction::Direct(tx) => tx,
        }
    }
}
