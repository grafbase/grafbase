use serde::de::Error;
use wrapping::Wrapping;

use crate::ids::MetaTypeId;

use super::MetaFieldTypeRecord;

impl serde::Serialize for MetaFieldTypeRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.wrappers.len() == 0 && !self.wrappers.inner_is_required() {
            self.target.serialize(serializer)
        } else {
            (self.target, self.wrappers).serialize(serializer)
        }
    }
}

impl<'de> serde::Deserialize<'de> for MetaFieldTypeRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // This is only going to work for self describing formats
        let (wrappers, target) = deserializer.deserialize_any(Visitor)?;

        Ok(MetaFieldTypeRecord { wrappers, target })
    }
}

struct Visitor;
impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value = (Wrapping, MetaTypeId);

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a single integer or a pair of integers")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let target = seq.next_element::<MetaTypeId>()?;
        let wrappers = seq.next_element::<Wrapping>()?;
        let done = seq.next_element::<u64>()?;

        if target.is_none() || wrappers.is_none() || done.is_some() {
            return Err(A::Error::custom("Malformed field record list"));
        }
        Ok((wrappers.unwrap(), target.unwrap()))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok((
            Wrapping::nullable(),
            // I am not happy about this - 1 but it's the easiest solution :/
            MetaTypeId::new((v - 1) as usize),
        ))
    }
}
