use crate::NodeID;
use getrandom::getrandom;
use internment::ArcIntern;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash)]
/// A node identifier within a particular [`Tree`].
///
/// This ID is used to get [`Node`] references from an [`Tree`].
/// Cheap to clone.
pub enum ResponseNodeId {
    // TODO: For compaction, it could be interesting to have a counter of reference for this ID,
    // (so let's switch the id: u64 by an ArcIntern)
    /// An ID which describe an Internal Node which isn't an Entity, like a primitive.
    Internal {
        id: u64,
    },
    // An ID which describe an Entity
    NodeID(ArcIntern<String>),
}

impl Default for ResponseNodeId {
    fn default() -> Self {
        Self::internal()
    }
}

impl ResponseNodeId {
    /// Generate a new Internal NodeID
    pub fn internal() -> Self {
        let mut buf = [0u8; 8];
        getrandom(&mut buf[..]).expect("Shouldn't fail");

        Self::Internal {
            id: u64::from_be_bytes(buf),
        }
    }

    pub fn node<'a, S: AsRef<NodeID<'a>>>(id: S) -> Self {
        Self::NodeID(ArcIntern::new(id.as_ref().to_string()))
    }

    pub fn get_node_id(&self) -> Option<ArcIntern<String>> {
        if let Self::NodeID(arc) = self {
            Some(arc.clone())
        } else {
            None
        }
    }
}

impl serde::Serialize for ResponseNodeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ResponseNodeId::Internal { id } => serializer.serialize_u64(*id),
            ResponseNodeId::NodeID(string) => serializer.serialize_str(string.as_ref()),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ResponseNodeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor {}

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ResponseNodeId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a u64 or string")
            }

            fn visit_u64<E>(self, id: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ResponseNodeId::Internal { id })
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ResponseNodeId::NodeID(ArcIntern::new(value.to_string())))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ResponseNodeId::NodeID(ArcIntern::new(value)))
            }
        }

        // This does rule out non self-describing formats :(
        deserializer.deserialize_any(Visitor {})
    }
}

// TODO: Serde tests
