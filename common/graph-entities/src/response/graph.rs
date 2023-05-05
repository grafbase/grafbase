use crate::NodeID;
use internment::ArcIntern;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
/// A node identifier within a particular [`Tree`].
///
/// This ID is used to get [`Node`] references from an [`Tree`].
/// Cheap to clone.
pub struct ResponseNodeId(pub(crate) u32);

pub trait ResponseIdLookup {
    fn node_id(&self) -> Option<ArcIntern<String>>;
    fn lookup_actual_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId>;
}

impl ResponseIdLookup for ResponseNodeId {
    fn node_id(&self) -> Option<ArcIntern<String>> {
        None
    }

    fn lookup_actual_id(&self, _response: &super::QueryResponse) -> Option<ResponseNodeId> {
        Some(*self)
    }
}

impl ResponseIdLookup for NodeID<'_> {
    fn node_id(&self) -> Option<ArcIntern<String>> {
        Some(ArcIntern::new(self.as_ref().to_string()))
    }

    fn lookup_actual_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId> {
        response.entity_ids.get(&self.node_id().unwrap()).copied()
    }
}

impl ResponseIdLookup for ArcIntern<String> {
    fn node_id(&self) -> Option<ArcIntern<String>> {
        Some(self.clone())
    }

    fn lookup_actual_id(&self, response: &super::QueryResponse) -> Option<ResponseNodeId> {
        response.entity_ids.get(self).copied()
    }
}

/*
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

        deserializer.deserialize_any(Visitor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_node_id_serde() {
        let response_id = ResponseNodeId::Internal { id: u64::MAX };
        assert_eq!(
            serde_json::from_str::<ResponseNodeId>(&serde_json::to_string(&response_id).unwrap()).unwrap(),
            response_id
        );

        let response_id = ResponseNodeId::NodeID(ArcIntern::new("I-am-an-id".into()));
        assert_eq!(
            serde_json::from_str::<ResponseNodeId>(&serde_json::to_string(&response_id).unwrap()).unwrap(),
            response_id
        );
    }
}
 */
