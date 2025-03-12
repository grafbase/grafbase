use crate::state::WasiState;

pub use super::grafbase::sdk::token::*;

impl Host for WasiState {}

impl From<TokenResult> for runtime::extension::Token {
    fn from(value: TokenResult) -> Self {
        match value {
            TokenResult::Anonymous => Self::Anonymous,
            TokenResult::Bytes(items) => Self::Bytes(items),
        }
    }
}
