use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

const NULL: serde_json::Value = serde_json::Value::Null;

// TODO: Hash is only used to generate a cache key for engine. To be removed once moved to the
// new cache key.
// TODO: remove Clone with gateway refactor...
#[derive(Clone, Hash, serde::Serialize, serde::Deserialize)]
pub enum AccessToken {
    Anonymous,
    Jwt(JwtToken),
}

/// Represents an *arbitrary* JWT token. It's only guaranteed to have been validated
/// according to auth config, but there is no guarantee on the claims content.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct JwtToken {
    /// Claims can be empty.
    pub claims: HashMap<String, serde_json::Value>,
    /// Keeping the signature for faster hashing/cache key generation.
    /// Ordering matters which isn't necessarily ideal, but that's something we can improve upon
    /// later if necessary.
    pub signature: Vec<u8>,
}

impl Hash for JwtToken {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.signature.hash(state);
    }
}

impl AccessToken {
    pub fn stable_id(&self) -> u8 {
        match self {
            AccessToken::Anonymous => 0,
            AccessToken::Jwt(_) => 1,
        }
    }

    pub fn is_anonymous(&self) -> bool {
        matches!(self, AccessToken::Anonymous)
    }

    pub fn get_claim(&self, key: &str) -> &serde_json::Value {
        match self {
            AccessToken::Anonymous => &NULL,
            AccessToken::Jwt(token) => token.claims.get(key).unwrap_or(&NULL),
        }
    }

    pub fn get_claim_with_path(&self, path: &[String]) -> &serde_json::Value {
        let mut path = path.iter();
        let Some(root) = path.next() else {
            return &NULL;
        };
        let parent = self.get_claim(root);
        path.fold(parent, |parent, key| {
            if let Some(object) = parent.as_object() {
                object.get(key).unwrap_or(&NULL)
            } else {
                &NULL
            }
        })
    }
}
