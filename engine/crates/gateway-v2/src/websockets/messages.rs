//! Message definitions for the [GraphQLOverWebsocket protocol][1]
//!
//! [1]: https://github.com/graphql/graphql-over-http/blob/main/rfcs/GraphQLOverWebSocket.md

use std::collections::HashMap;

#[derive(serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    ConnectionInit {
        #[serde(default)]
        payload: InitPayload,
    },
    Subscribe {
        id: String,
        payload: engine_v2::Request,
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
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

#[derive(serde::Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Next {
        id: String,
        payload: engine_v2::Response,
    },
    Error {
        id: String,
        payload: engine_v2::Response,
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
