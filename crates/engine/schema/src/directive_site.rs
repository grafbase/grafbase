use crate::{
    CompositeType, CompositeTypeId, Definition, DefinitionId, DirectiveSite, DirectiveSiteId, EntityDefinition,
    EntityDefinitionId,
};

impl std::fmt::Display for DirectiveSite<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectiveSite::Scalar(scalar) => f.write_str(scalar.name()),
            DirectiveSite::Object(object) => f.write_str(object.name()),
            DirectiveSite::Interface(interface) => f.write_str(interface.name()),
            DirectiveSite::Union(union) => f.write_str(union.name()),
            DirectiveSite::Enum(enm) => f.write_str(enm.name()),
            DirectiveSite::InputObject(input_object) => f.write_str(input_object.name()),
            DirectiveSite::Field(field) => write!(f, "{}.{}", field.parent_entity().name(), field.name()),
        }
    }
}

impl From<DefinitionId> for DirectiveSiteId {
    fn from(definition: DefinitionId) -> Self {
        match definition {
            DefinitionId::Scalar(id) => DirectiveSiteId::Scalar(id),
            DefinitionId::Object(id) => DirectiveSiteId::Object(id),
            DefinitionId::Interface(id) => DirectiveSiteId::Interface(id),
            DefinitionId::Union(id) => DirectiveSiteId::Union(id),
            DefinitionId::Enum(id) => DirectiveSiteId::Enum(id),
            DefinitionId::InputObject(id) => DirectiveSiteId::InputObject(id),
        }
    }
}

impl<'a> From<Definition<'a>> for DirectiveSite<'a> {
    fn from(definition: Definition<'a>) -> Self {
        match definition {
            Definition::Scalar(def) => DirectiveSite::Scalar(def),
            Definition::Object(def) => DirectiveSite::Object(def),
            Definition::Interface(def) => DirectiveSite::Interface(def),
            Definition::Union(def) => DirectiveSite::Union(def),
            Definition::Enum(def) => DirectiveSite::Enum(def),
            Definition::InputObject(def) => DirectiveSite::InputObject(def),
        }
    }
}

impl From<CompositeTypeId> for DirectiveSiteId {
    fn from(composite: CompositeTypeId) -> Self {
        match composite {
            CompositeTypeId::Object(id) => DirectiveSiteId::Object(id),
            CompositeTypeId::Interface(id) => DirectiveSiteId::Interface(id),
            CompositeTypeId::Union(id) => DirectiveSiteId::Union(id),
        }
    }
}

impl<'a> From<CompositeType<'a>> for DirectiveSite<'a> {
    fn from(composite: CompositeType<'a>) -> Self {
        match composite {
            CompositeType::Object(composite) => DirectiveSite::Object(composite),
            CompositeType::Interface(composite) => DirectiveSite::Interface(composite),
            CompositeType::Union(composite) => DirectiveSite::Union(composite),
        }
    }
}

impl From<EntityDefinitionId> for DirectiveSiteId {
    fn from(entity: EntityDefinitionId) -> Self {
        match entity {
            EntityDefinitionId::Object(id) => DirectiveSiteId::Object(id),
            EntityDefinitionId::Interface(id) => DirectiveSiteId::Interface(id),
        }
    }
}

impl<'a> From<EntityDefinition<'a>> for DirectiveSite<'a> {
    fn from(entity: EntityDefinition<'a>) -> Self {
        match entity {
            EntityDefinition::Object(entity) => DirectiveSite::Object(entity),
            EntityDefinition::Interface(entity) => DirectiveSite::Interface(entity),
        }
    }
}

impl From<DirectiveSiteId> for u32 {
    fn from(id: DirectiveSiteId) -> Self {
        const SHIFT: u32 = 3;
        match id {
            DirectiveSiteId::Scalar(id) => u32::from(id) << SHIFT,
            DirectiveSiteId::Object(id) => 0x1 | (u32::from(id) << SHIFT),
            DirectiveSiteId::Interface(id) => 0x2 | (u32::from(id) << SHIFT),
            DirectiveSiteId::Union(id) => 0x3 | (u32::from(id) << SHIFT),
            DirectiveSiteId::Enum(id) => 0x4 | (u32::from(id) << SHIFT),
            DirectiveSiteId::InputObject(id) => 0x5 | (u32::from(id) << SHIFT),
            DirectiveSiteId::Field(id) => 0x6 | (u32::from(id) << SHIFT),
        }
    }
}
