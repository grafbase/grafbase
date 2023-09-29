use std::{error::Error, fmt::Display, future::Future, net::Ipv4Addr, pin::Pin};

use async_runtime::make_send_on_wasm;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone)]
pub struct Bridge {
    port: u16,
}

#[derive(Debug)]
pub(crate) enum BridgeError {
    Reqwest(reqwest::Error),
    UnexpectedResponseError(String),
}

impl Error for BridgeError {}

impl From<reqwest::Error> for BridgeError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::Reqwest(error) => write!(f, "reqwest Error: {error:?}"),
            BridgeError::UnexpectedResponseError(response) => write!(f, "Unexpected Response Error: {response:?}"),
        }
    }
}

impl Bridge {
    pub fn new(port: u16) -> Bridge {
        Bridge { port }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub(crate) fn request<B: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: B,
    ) -> Pin<Box<dyn Future<Output = Result<R, BridgeError>> + Send + '_>> {
        let url = format!("http://{}:{}/{endpoint}", Ipv4Addr::LOCALHOST, self.port);
        let request = reqwest::Client::new().post(url).json(&body);
        Box::pin(make_send_on_wasm(async move {
            let response = request.send().await?;
            let status = response.status();
            if status.is_success() {
                Ok(response.json().await?)
            } else {
                Err(BridgeError::UnexpectedResponseError(
                    response.text().await.unwrap_or(format!("Status: {status}")),
                ))
            }
        }))
    }
}
