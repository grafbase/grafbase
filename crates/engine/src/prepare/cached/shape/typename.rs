use operation::{Location, PositionedResponseKey};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct TypenameShapeRecord {
    pub key: PositionedResponseKey,
    pub location: Location,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct TypenameShapeId(u32);
