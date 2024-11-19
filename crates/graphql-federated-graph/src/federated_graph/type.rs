use super::{EntityDefinitionId, InputObjectId, InterfaceId, ObjectId, TypeDefinitionId, UnionId, Wrapping};

#[derive(Clone, PartialEq, Eq, PartialOrd, Debug)]
pub struct Type {
    pub wrapping: Wrapping,
    pub definition: Definition,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Definition {
    Scalar(TypeDefinitionId),
    Object(ObjectId),
    Interface(InterfaceId),
    Union(UnionId),
    Enum(TypeDefinitionId),
    InputObject(InputObjectId),
}

impl Definition {
    pub fn as_enum(&self) -> Option<TypeDefinitionId> {
        if let Self::Enum(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_entity(&self) -> Option<EntityDefinitionId> {
        match self {
            Self::Object(id) => Some(EntityDefinitionId::Object(*id)),
            Self::Interface(id) => Some(EntityDefinitionId::Interface(*id)),
            _ => None,
        }
    }
}
