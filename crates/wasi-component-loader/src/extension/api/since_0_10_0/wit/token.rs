use crate::state::InstanceState;

pub use super::grafbase::sdk::token::*;

impl Host for InstanceState {}

impl From<TokenResult> for runtime::extension::Token {
    fn from(value: TokenResult) -> Self {
        match value {
            TokenResult::Anonymous => Self::Anonymous,
            TokenResult::Bytes(items) => Self::Bytes(items),
        }
    }
}
