mod access_log;
mod cache;
mod headers;
mod http_client;
mod nats;
mod shared_context;

use super::wit::*;
use crate::state::WasiState;

impl Host for WasiState {}

impl From<extension_catalog::KindDiscriminants> for ExtensionType {
    fn from(value: extension_catalog::KindDiscriminants) -> Self {
        match value {
            extension_catalog::KindDiscriminants::FieldResolver => ExtensionType::Resolver,
            extension_catalog::KindDiscriminants::Authenticator => ExtensionType::Authentication,
        }
    }
}
