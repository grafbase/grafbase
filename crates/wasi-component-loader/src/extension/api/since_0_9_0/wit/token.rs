pub use super::grafbase::sdk::token::*;

impl Host for crate::WasiState {}

impl From<Token> for runtime::extension::Token {
    fn from(value: Token) -> Self {
        match value {
            Token::Anonymous => Self::Anonymous,
            Token::Bytes(items) => Self::Bytes(items),
        }
    }
}
