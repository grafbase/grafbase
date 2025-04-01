use minicbor_serde::{
    Serializer,
    error::{DecodeError, EncodeError},
};
use serde::Serialize;

/// Serialise a type implementing [`serde::Serialize`] and return the encoded byte vector.
/// Please use this instead of `minicbor_serde::to_vec` due to this function serializing nulls correctly.
pub(crate) fn to_vec<T: Serialize>(val: T) -> Result<Vec<u8>, EncodeError<core::convert::Infallible>> {
    let mut serialized = Vec::new();
    let mut serializer = Serializer::new(&mut serialized);

    // Necessary for serde_json::Value which serializes `Null` as unit rather than none...
    serializer.serialize_unit_as_null(true);
    val.serialize(&mut serializer)?;

    Ok(serialized)
}

// for consistency and convenience.
pub(crate) fn from_slice<'de, T: serde::Deserialize<'de>>(data: &'de [u8]) -> Result<T, DecodeError> {
    let mut deserializer = minicbor_serde::Deserializer::new(data);
    T::deserialize(&mut deserializer)
}

// for consistency and convenience.
pub(crate) fn from_slice_with_seed<'de, Seed: serde::de::DeserializeSeed<'de>>(
    data: &'de [u8],
    seed: Seed,
) -> Result<Seed::Value, DecodeError> {
    let mut deserializer = minicbor_serde::Deserializer::new(data);
    seed.deserialize(&mut deserializer)
}
