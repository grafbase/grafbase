use std::future::Future;

use error::ErrorResponse;
use extension_catalog::ExtensionId;

pub trait AuthenticationExtension<Context: Send + Sync + 'static>: Send + Sync + 'static {
    fn authenticate(
        &self,
        ctx: &Context,
        extension_ids: &[ExtensionId],
        gateway_headers: http::HeaderMap,
    ) -> impl Future<Output = (http::HeaderMap, Result<Token, ErrorResponse>)> + Send;
}

#[derive(Clone, Debug)]
pub enum Token {
    Anonymous,
    Bytes(Vec<u8>),
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
            TokenRef::Bytes(bytes) => Token::Bytes(bytes.to_vec()),
        }
    }
}
