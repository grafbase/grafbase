use super::{EntityDefinitionId, EnumDefinitionId, InputObjectId, InterfaceId, ObjectId, ScalarDefinitionId, UnionId};
use wrapping::Wrapping;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug)]
pub(crate) struct Type {
    pub wrapping: Wrapping,
    pub definition: Definition,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Definition {
    Scalar(ScalarDefinitionId),
    Object(ObjectId),
    Interface(InterfaceId),
    Union(UnionId),
    Enum(EnumDefinitionId),
    InputObject(InputObjectId),
}

impl Definition {
    pub fn as_enum(&self) -> Option<EnumDefinitionId> {
        if let Self::Enum(v) = self { Some(*v) } else { None }
    }

    pub fn as_entity(&self) -> Option<EntityDefinitionId> {
        match self {
            Self::Object(id) => Some(EntityDefinitionId::Object(*id)),
            Self::Interface(id) => Some(EntityDefinitionId::Interface(*id)),
            _ => None,
        }
    }
}
