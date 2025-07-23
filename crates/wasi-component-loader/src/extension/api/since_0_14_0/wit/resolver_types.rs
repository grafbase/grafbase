use crate::state::InstanceState;

pub use super::grafbase::sdk::resolver_types::*;

impl Host for InstanceState {}

impl From<Data> for runtime::extension::Data {
    fn from(data: Data) -> Self {
        match data {
            Data::Json(bytes) => runtime::extension::Data::Json(bytes.into()),
            Data::Cbor(bytes) => runtime::extension::Data::Cbor(bytes.into()),
        }
    }
}
