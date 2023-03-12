use std::{error::Error, fmt::Display, net::Ipv4Addr};

use serde::{de::DeserializeOwned, Serialize};

pub(crate) struct Bridge {
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
    pub(crate) fn new(port: u16) -> Bridge {
        Bridge { port }
    }

    pub(crate) async fn request<B: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: B,
    ) -> Result<R, BridgeError> {
        let url = format!("http://{}:{}{endpoint}", Ipv4Addr::LOCALHOST, self.port);
        let response = reqwest::Client::new().post(url).json(&body).send().await?;
        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            Err(BridgeError::UnexpectedResponseError(
                response.text().await.unwrap_or(format!("Status: {}", status)),
            ))
        }
    }
}
