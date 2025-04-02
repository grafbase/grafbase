use crate::{
    CompositeType, CompositeTypeId, DirectiveSite, DirectiveSiteId, EntityDefinition, EntityDefinitionId,
    TypeDefinition, TypeDefinitionId,
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
            DirectiveSite::EnumValue(value) => write!(f, "{}.{}", value.parent_enum().name(), value.name()),
            // Not a great Display...
            DirectiveSite::InputValue(value) => write!(f, "{}", value.name()),
        }
    }
}

impl From<TypeDefinitionId> for DirectiveSiteId {
    fn from(definition: TypeDefinitionId) -> Self {
        match definition {
            TypeDefinitionId::Scalar(id) => DirectiveSiteId::Scalar(id),
            TypeDefinitionId::Object(id) => DirectiveSiteId::Object(id),
            TypeDefinitionId::Interface(id) => DirectiveSiteId::Interface(id),
            TypeDefinitionId::Union(id) => DirectiveSiteId::Union(id),
            TypeDefinitionId::Enum(id) => DirectiveSiteId::Enum(id),
            TypeDefinitionId::InputObject(id) => DirectiveSiteId::InputObject(id),
        }
    }
}

impl<'a> From<TypeDefinition<'a>> for DirectiveSite<'a> {
    fn from(definition: TypeDefinition<'a>) -> Self {
        match definition {
            TypeDefinition::Scalar(def) => DirectiveSite::Scalar(def),
            TypeDefinition::Object(def) => DirectiveSite::Object(def),
            TypeDefinition::Interface(def) => DirectiveSite::Interface(def),
            TypeDefinition::Union(def) => DirectiveSite::Union(def),
            TypeDefinition::Enum(def) => DirectiveSite::Enum(def),
            TypeDefinition::InputObject(def) => DirectiveSite::InputObject(def),
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
