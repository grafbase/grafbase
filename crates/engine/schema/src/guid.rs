/// GUIDs for schema elements. They're guaranteed to be unique across the schema, mostly useful when sharing them with Wasm extensions to simply
/// send a u32.
use crate::{
    DirectiveSiteId, EnumDefinitionId, EnumValueId, FieldDefinitionId, InputObjectDefinitionId, InputValueDefinitionId,
    InterfaceDefinitionId, ObjectDefinitionId, ScalarDefinitionId, TypeDefinitionId, UnionDefinitionId,
};

const SHIFT: u32 = 4;

impl DirectiveSiteId {
    pub fn as_guid(self) -> u32 {
        match self {
            DirectiveSiteId::Scalar(id) => id.as_guid(),
            DirectiveSiteId::Object(id) => id.as_guid(),
            DirectiveSiteId::Interface(id) => id.as_guid(),
            DirectiveSiteId::Union(id) => id.as_guid(),
            DirectiveSiteId::Enum(id) => id.as_guid(),
            DirectiveSiteId::InputObject(id) => id.as_guid(),
            DirectiveSiteId::Field(id) => id.as_guid(),
            DirectiveSiteId::EnumValue(id) => id.as_guid(),
            DirectiveSiteId::InputValue(id) => id.as_guid(),
        }
    }
}

impl TypeDefinitionId {
    pub fn as_guid(self) -> u32 {
        match self {
            TypeDefinitionId::Scalar(id) => id.as_guid(),
            TypeDefinitionId::Object(id) => id.as_guid(),
            TypeDefinitionId::Interface(id) => id.as_guid(),
            TypeDefinitionId::Union(id) => id.as_guid(),
            TypeDefinitionId::Enum(id) => id.as_guid(),
            TypeDefinitionId::InputObject(id) => id.as_guid(),
        }
    }
}

impl ScalarDefinitionId {
    pub fn as_guid(self) -> u32 {
        u32::from(self) << SHIFT
    }
}

impl ObjectDefinitionId {
    pub fn as_guid(self) -> u32 {
        0x1 | (u32::from(self) << SHIFT)
    }
}

impl InterfaceDefinitionId {
    pub fn as_guid(self) -> u32 {
        0x2 | (u32::from(self) << SHIFT)
    }
}

impl UnionDefinitionId {
    pub fn as_guid(self) -> u32 {
        0x3 | (u32::from(self) << SHIFT)
    }
}

impl EnumDefinitionId {
    pub fn as_guid(self) -> u32 {
        0x4 | (u32::from(self) << SHIFT)
    }
}

impl InputObjectDefinitionId {
    pub fn as_guid(self) -> u32 {
        0x5 | (u32::from(self) << SHIFT)
    }
}

impl FieldDefinitionId {
    pub fn as_guid(self) -> u32 {
        0x6 | (u32::from(self) << SHIFT)
    }
}

impl EnumValueId {
    pub fn as_guid(self) -> u32 {
        0x7 | (u32::from(self) << SHIFT)
    }
}

impl InputValueDefinitionId {
    pub fn as_guid(self) -> u32 {
        0x8 | (u32::from(self) << SHIFT)
    }
}
