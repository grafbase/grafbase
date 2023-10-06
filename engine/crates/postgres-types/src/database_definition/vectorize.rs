use std::iter::FromIterator;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<'a, T, K, V, S>(target: T, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: IntoIterator<Item = (&'a K, &'a V)>,
    K: Serialize + 'a,
    V: Serialize + 'a,
{
    ser.collect_seq(target)
}

pub fn deserialize<'de, T, K, V, D>(des: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + FromIterator<(K, V)>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    let container: Vec<_> = serde::Deserialize::deserialize(des)?;
    Ok(container.into_iter().collect::<T>())
}
