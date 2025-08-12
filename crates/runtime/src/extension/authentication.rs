use std::{future::Future, sync::Arc};

use error::ErrorResponse;
use event_queue::EventQueue;
use extension_catalog::ExtensionId;

pub trait AuthenticationExtension: Send + Sync + 'static {
    fn authenticate(
        &self,
        event_queue: &Arc<EventQueue>,
        hooks_context: &Arc<[u8]>,
        gateway_headers: http::HeaderMap,
        ids: &[ExtensionId],
    ) -> impl Future<Output = (http::HeaderMap, Result<Token, ErrorResponse>)> + Send;

    fn public_metadata_endpoints(&self) -> impl Future<Output = Result<Vec<PublicMetadataEndpoint>, String>> + Send;
}

pub struct PublicMetadataEndpoint {
    pub path: String,
    pub response_body: Vec<u8>,
    pub headers: http::HeaderMap,
}

#[derive(Clone, Debug)]
pub enum Token {
    Anonymous,
    // Arc for Wasmtime because we can't return an non 'static value from a function.
    Bytes(Arc<[u8]>),
}

impl Token {
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Token::Anonymous => None,
            Token::Bytes(bytes) => Some(bytes),
        }
    }

    pub fn as_ref(&self) -> TokenRef<'_> {
        match self {
            Token::Anonymous => TokenRef::Anonymous,
            Token::Bytes(bytes) => TokenRef::Bytes(bytes),
        }
    }
}

#[derive(Clone, Copy)]
pub enum TokenRef<'a> {
    Anonymous,
    Bytes(&'a [u8]),
}

impl TokenRef<'_> {
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            TokenRef::Anonymous => None,
            TokenRef::Bytes(bytes) => Some(bytes),
        }
    }

    pub fn to_owned(&self) -> Token {
        match self {
            TokenRef::Anonymous => Token::Anonymous,
            TokenRef::Bytes(bytes) => Token::Bytes(bytes.to_vec().into()),
        }
    }
}
