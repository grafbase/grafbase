use std::{collections::HashMap, future::Future};

use error::ErrorResponse;
use serde_json::Value;

use crate::extension::{Token, TokenRef};

#[derive(Clone, Debug)]
pub enum LegacyToken {
    Anonymous,
    Jwt(JwtToken),
    Extension(Token),
}

/// Represents an *arbitrary* JWT token. It's only guaranteed to have been validated
/// according to auth config, but there is no guarantee on the claims content.
#[derive(Clone, Debug)]
pub struct JwtToken {
    /// Claims can be empty.
    pub claims: HashMap<String, Value>,
    pub bytes: Vec<u8>,
}

impl LegacyToken {
    pub fn is_anonymous(&self) -> bool {
        matches!(self, LegacyToken::Anonymous | LegacyToken::Extension(Token::Anonymous))
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            LegacyToken::Anonymous => None,
            LegacyToken::Jwt(jwt) => Some(&jwt.bytes),
            LegacyToken::Extension(token) => token.as_bytes(),
        }
    }

    pub fn as_ref(&self) -> TokenRef<'_> {
        match self {
            LegacyToken::Anonymous => TokenRef::Anonymous,
            LegacyToken::Jwt(jwt) => TokenRef::Bytes(&jwt.bytes),
            LegacyToken::Extension(token) => token.as_ref(),
        }
    }

    pub fn get_claim(&self, key: &str) -> Option<&Value> {
        match self {
            LegacyToken::Anonymous => None,
            LegacyToken::Jwt(token) => token.claims.get(key),
            LegacyToken::Extension(_) => None,
        }
    }

    pub fn get_claim_with_path(&self, path: &[String]) -> &Value {
        let mut path = path.iter();
        let Some(root) = path.next() else {
            return &Value::Null;
        };
        let parent = self.get_claim(root).unwrap_or(&Value::Null);
        path.fold(parent, |parent, key| {
            if let Some(object) = parent.as_object() {
                object.get(key).unwrap_or(&Value::Null)
            } else {
                &Value::Null
            }
        })
    }
}

pub trait Authenticate<Context> {
    fn authenticate(
        &self,
        context: &Context,
        headers: http::HeaderMap,
    ) -> impl Future<Output = Result<(http::HeaderMap, LegacyToken), ErrorResponse>> + Send;
}
