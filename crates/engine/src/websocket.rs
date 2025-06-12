//! Message definitions for the [GraphQLOverWebsocket protocol][1]
//!
//! [1]: https://github.com/graphql/graphql-over-http/blob/main/rfcs/GraphQLOverWebSocket.md

use operation::Request;

#[derive(serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    ConnectionInit {
        #[serde(default)]
        payload: InitPayload,
    },
    Subscribe(SubscribeEvent),
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

#[derive(serde::Deserialize)]
pub struct SubscribeEvent {
    pub id: String,
    pub payload: RequestPayload,
}

#[derive(serde::Deserialize, Debug)]
pub struct RequestPayload(pub(crate) Request);

#[derive(Debug, Default, serde::Deserialize)]
pub struct InitPayload(pub(crate) Option<serde_json::Map<String, serde_json::Value>>);

#[derive(serde::Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", bound = "")]
pub enum Message {
    Next {
        id: String,
        payload: ResponsePayload,
    },
    Error {
        id: String,
        payload: ResponsePayload,
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

#[derive(serde::Serialize)]
#[serde(bound = "")]
pub struct ResponsePayload(pub(super) crate::response::Response);

impl std::fmt::Debug for ResponsePayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponsePayload").finish_non_exhaustive()
    }
}

impl Message {
    pub fn close(code: u16, reason: impl Into<String>) -> Self {
        Self::Close {
            code,
            reason: reason.into(),
        }
    }
}
