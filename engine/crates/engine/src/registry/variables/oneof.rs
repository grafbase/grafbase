use std::{fmt, marker::PhantomData};

use serde::{
    de::{Error, MapAccess, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug, Clone)]
pub struct OneOf<T> {
    pub name: String,
    pub value: T,
}

impl<T: PartialEq> PartialEq for OneOf<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value == other.value
    }
}

impl<T: Eq> Eq for OneOf<T> {}

struct OneOfVisitor<T> {
    mark: PhantomData<fn() -> OneOf<T>>,
}

impl<T> OneOfVisitor<T> {
    fn new() -> Self {
        OneOfVisitor { mark: PhantomData }
    }
}

impl<'de, T> Visitor<'de> for OneOfVisitor<T>
where
    T: Deserialize<'de>,
{
    // The type that our Visitor is going to produce.
    type Value = OneOf<T>;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a very special map")
    }

    // Deserialize MyMap from an abstract "map" provided by the
    // Deserializer. The MapAccess input is a callback provided by
    // the Deserializer to let us see each entry in the map.
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
        M::Error: Error,
    {
        // While there are entries remaining in the input, add them
        // into our map.
        if let Some((name, value)) = access.next_entry::<String, T>()? {
            if let Some((other, _)) = access.next_entry::<String, T>()? {
                Err(M::Error::custom(format!(
                    "Expected at most one field for @oneof, found: {name} and {other}"
                )))
            } else {
                Ok(OneOf { name, value })
            }
        } else {
            Err(M::Error::custom("Expected at least one field for @oneof"))
        }
    }
}

// This is the trait that informs Serde how to deserialize MyMap.
impl<'de, T> Deserialize<'de> for OneOf<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_map(OneOfVisitor::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::panic)]
    #[test]
    fn test_deserializer() {
        assert_eq!(
            serde_json::from_str::<OneOf<String>>(r#"{"key": "value"}"#).unwrap(),
            OneOf {
                name: "key".to_string(),
                value: "value".to_string()
            }
        );

        let result = serde_json::from_str::<OneOf<String>>(r#"{"key": "value", "key2": "value"}"#);
        let Err(err) = result else { panic!("Expected an error.") };
        assert!(
            err.to_string().contains("at most one field"),
            "Unexpected message: {err}"
        );

        let result = serde_json::from_str::<OneOf<String>>(r"{}");
        let Err(err) = result else { panic!("Expected an error.") };
        assert!(
            err.to_string().contains("at least one field"),
            "Unexpected message: {err}"
        );
    }
}
