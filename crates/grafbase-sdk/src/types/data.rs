use serde::Serialize;

use crate::{SdkError, cbor, wit};

/// Data serialized in either JSON or CBOR
#[derive(Debug)]
pub enum Data {
    /// JSON bytes
    Json(Vec<u8>),
    /// CBOR bytes
    Cbor(Vec<u8>),
}

impl Data {
    /// Serialize a type into the most efficient supported serialization
    pub fn new<T: Serialize>(data: T) -> Result<Self, SdkError> {
        let bytes = cbor::to_vec(&data)?;
        Ok(Data::Cbor(bytes))
    }
}

impl From<Data> for wit::Data {
    fn from(value: Data) -> Self {
        match value {
            Data::Json(bytes) => wit::Data::Json(bytes),
            Data::Cbor(bytes) => wit::Data::Cbor(bytes),
        }
    }
}
