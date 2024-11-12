use serde::{de::Visitor, Deserializer};
use size::Size;

pub(crate) fn deserialize_positive_size<'de, D>(deserializer: D) -> Result<Size, D::Error>
where
    D: Deserializer<'de>,
{
    struct SizeVisitor;

    impl Visitor<'_> for SizeVisitor {
        type Value = Size;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a size value as string or number of bytes")
        }

        fn visit_str<E>(self, value: &str) -> Result<Size, E>
        where
            E: serde::de::Error,
        {
            Size::from_str(value).map_err(serde::de::Error::custom)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Size::from_bytes(v))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Size::from_bytes(v))
        }
    }

    let size = deserializer.deserialize_any(SizeVisitor)?;
    if size.bytes() < 0 {
        Err(serde::de::Error::custom("size must be positive"))
    } else {
        Ok(size)
    }
}
