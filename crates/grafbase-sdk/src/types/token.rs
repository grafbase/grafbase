use crate::wit;

/// Token produced by an authentication extension.
#[derive(Clone)]
pub struct Token(wit::Token);

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token").finish_non_exhaustive()
    }
}

impl From<wit::Token> for Token {
    fn from(token: wit::Token) -> Self {
        Self(token)
    }
}

impl From<Token> for wit::Token {
    fn from(token: Token) -> Self {
        token.0
    }
}

impl Token {
    /// Create a new anonymous token.
    pub fn anonymous() -> Self {
        Self(wit::Token::Anonymous)
    }

    /// Create a new token from raw bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(wit::Token::Bytes(bytes))
    }

    /// Whether the current user is anonymous or not.
    pub fn is_anonymous(&self) -> bool {
        matches!(self.0, wit::Token::Anonymous)
    }

    /// Get the token's raw bytes. Will be `None` if the user is anonymous.
    pub fn into_bytes(self) -> Option<Vec<u8>> {
        match self.0 {
            wit::Token::Anonymous => None,
            wit::Token::Bytes(bytes) => Some(bytes),
        }
    }

    /// Get a reference to the token's raw bytes. Will be `None` if the user is anonymous.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match &self.0 {
            wit::Token::Anonymous => None,
            wit::Token::Bytes(bytes) => Some(bytes),
        }
    }
}
