use crate::InstanceState;

pub use super::grafbase::sdk::token::*;

impl Host for InstanceState {}

impl From<Token> for runtime::extension::Token {
    fn from(token: Token) -> Self {
        match token {
            Token::Anonymous => runtime::extension::Token::Anonymous,
            Token::Bytes(bytes) => runtime::extension::Token::Bytes(bytes),
        }
    }
}
