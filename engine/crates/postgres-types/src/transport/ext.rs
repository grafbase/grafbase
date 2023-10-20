use async_trait::async_trait;
use futures::{pin_mut, StreamExt};
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::Transport;

impl<T: ?Sized> TransportExt for T where T: Transport {}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait TransportExt: Transport {
    async fn collect_query<T>(&self, query: &str, params: Vec<Value>) -> crate::Result<Vec<T>>
    where
        T: DeserializeOwned + Send,
    {
        let mut result = Vec::new();
        let stream = self.parameterized_query(query, params);
        pin_mut!(stream);

        while let Some(value) = stream.next().await {
            result.push(serde_json::from_value(value?).unwrap());
        }

        Ok(result)
    }
}
