use std::{error::Error, fmt::Display, net::Ipv4Addr};

use serde::{de::DeserializeOwned, Serialize};

pub(crate) struct Bridge {
    port: u16,
}

#[derive(Debug)]
pub(crate) enum BridgeError {
    Surf(surf::Error),
    UnexpectedResponseError(String),
}

impl Error for BridgeError {}

impl From<surf::Error> for BridgeError {
    fn from(value: surf::Error) -> Self {
        Self::Surf(value)
    }
}

impl Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::Surf(error) => write!(f, "Surf Error: {error:?}"),
            BridgeError::UnexpectedResponseError(response) => write!(f, "Unexpected Response Error: {response:?}"),
        }
    }
}

impl Bridge {
    pub(crate) fn new(port: u16) -> Bridge {
        Bridge { port }
    }

    pub(crate) async fn request<B: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: B,
    ) -> Result<R, BridgeError> {
        let url = format!("http://{}:{}{endpoint}", Ipv4Addr::LOCALHOST, self.port);
        let mut response = surf::client().post(url).body_json(&body)?.await?;
        if response.status().is_success() {
            Ok(response.body_json().await?)
        } else {
            Err(BridgeError::UnexpectedResponseError(
                response
                    .body_string()
                    .await
                    .unwrap_or(format!("Status: {}", response.status())),
            ))
        }
    }
}
