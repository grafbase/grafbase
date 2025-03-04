use crate::extension::wit;

impl From<wit::Token> for runtime::extension::Token {
    fn from(token: wit::Token) -> Self {
        use runtime::extension::Token;
        match token {
            wit::Token::Anonymous => Token::Anonymous,
            wit::Token::Bytes(bytes) => Token::Bytes(bytes),
        }
    }
}

impl From<runtime::extension::Token> for wit::Token {
    fn from(token: runtime::extension::Token) -> Self {
        use wit::Token;
        match token {
            runtime::extension::Token::Anonymous => Token::Anonymous,
            runtime::extension::Token::Bytes(bytes) => Token::Bytes(bytes),
        }
    }
}
