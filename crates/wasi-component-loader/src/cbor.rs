use minicbor_serde::{Serializer, error::EncodeError};
use serde::Serialize;

/// Serialise a type implementing [`serde::Serialize`] and return the encoded byte vector.
/// Please use this instead of `minicbor_serde::to_vec` due to this function serializing nulls correctly.
pub fn to_vec<T: Serialize>(val: T) -> Result<Vec<u8>, EncodeError<core::convert::Infallible>> {
    let mut serialized = Vec::new();
    let mut serializer = Serializer::new(&mut serialized);

    serializer.serialize_unit_as_null(true);
    val.serialize(&mut serializer)?;

    Ok(serialized)
}
