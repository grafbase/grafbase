//! Message definitions for the [GraphQLOverWebsocket protocol][1]
//!
//! [1]: https://github.com/graphql/graphql-over-http/blob/main/rfcs/GraphQLOverWebSocket.md

use std::{borrow::Cow, collections::HashMap};

use engine::RequestExtensions;
use engine_v2_common::BatchGraphqlRequest;
use http::{HeaderName, HeaderValue};
use serde::Deserialize;

use crate::response::Response;

#[derive(serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event<'a> {
    ConnectionInit {
        #[serde(default)]
        payload: InitPayload,
    },
    Subscribe {
        id: String,
        #[serde(borrow)]
        payload: Box<BatchGraphqlRequest<'a, RequestExtensions>>,
    },
    Complete {
        id: String,
    },
    Ping {
        payload: Option<serde_json::Value>,
    },
    Pong {
        payload: Option<serde_json::Value>,
    },
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct InitPayload {
    #[serde(deserialize_with = "deserialize_as_hash_map")]
    pub headers: http::HeaderMap,
}

fn deserialize_as_hash_map<'de, D>(deserializer: D) -> Result<http::HeaderMap, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let map = HashMap::<Cow<'_, str>, Cow<'_, str>>::deserialize(deserializer)?;
    let mut headers = http::HeaderMap::new();
    for (key, value) in map {
        headers.insert(
            HeaderName::try_from(key.as_ref()).map_err(serde::de::Error::custom)?,
            HeaderValue::try_from(value.as_ref()).map_err(serde::de::Error::custom)?,
        );
    }
    Ok(headers)
}

#[derive(serde::Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Next {
        id: String,
        payload: Response,
    },
    Error {
        id: String,
        payload: Response,
    },
    Complete {
        id: String,
    },
    ConnectionAck {
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<serde_json::Value>,
    },
    Ping {
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<serde_json::Value>,
    },
    Pong {
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<serde_json::Value>,
    },
    Close {
        code: u16,
        reason: String,
    },
}

impl Message {
    pub fn close(code: u16, reason: impl Into<String>) -> Self {
        Self::Close {
            code,
            reason: reason.into(),
        }
    }
}
