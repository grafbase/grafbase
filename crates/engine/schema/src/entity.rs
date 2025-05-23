use walker::{Iter, Walk};

use crate::{
    CompositeType, CompositeTypeId, DeprecatedDirective, EntityDefinition, EntityDefinitionId, FieldDefinition,
    InterfaceDefinition, InterfaceDefinitionId, ObjectDefinitionId, TypeDefinitionId, TypeSystemDirective,
};

impl EntityDefinitionId {
    pub fn maybe_from(definition: TypeDefinitionId) -> Option<EntityDefinitionId> {
        match definition {
            TypeDefinitionId::Object(id) => Some(EntityDefinitionId::Object(id)),
            TypeDefinitionId::Interface(id) => Some(EntityDefinitionId::Interface(id)),
            _ => None,
        }
    }

    pub fn as_composite_type(self) -> CompositeTypeId {
        match self {
            EntityDefinitionId::Object(id) => CompositeTypeId::Object(id),
            EntityDefinitionId::Interface(id) => CompositeTypeId::Interface(id),
        }
    }
}

impl<'a> EntityDefinition<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            EntityDefinition::Object(item) => item.name(),
            EntityDefinition::Interface(item) => item.name(),
        }
    }

    pub fn description(&self) -> Option<&'a str> {
        match self {
            EntityDefinition::Object(item) => item.description(),
            EntityDefinition::Interface(item) => item.description(),
        }
    }

    pub fn has_deprecated(&self) -> Option<DeprecatedDirective<'a>> {
        match self {
            EntityDefinition::Object(item) => item.has_deprecated(),
            EntityDefinition::Interface(item) => item.has_deprecated(),
        }
    }

    pub fn as_composite_type(self) -> CompositeType<'a> {
        match self {
            EntityDefinition::Object(item) => CompositeType::Object(item),
            EntityDefinition::Interface(item) => CompositeType::Interface(item),
        }
    }

    pub fn directives(&self) -> impl Iter<Item = TypeSystemDirective<'a>> + 'a {
        let (schema, directive_ids) = match self {
            EntityDefinition::Object(item) => (item.schema, &item.as_ref().directive_ids),
            EntityDefinition::Interface(item) => (item.schema, &item.as_ref().directive_ids),
        };
        directive_ids.walk(schema)
    }

    pub fn fields(&self) -> impl Iter<Item = FieldDefinition<'a>> + 'a {
        let (schema, field_ids) = match self {
            EntityDefinition::Object(item) => (item.schema, &item.as_ref().field_ids),
            EntityDefinition::Interface(item) => (item.schema, &item.as_ref().field_ids),
        };
        field_ids.walk(schema)
    }

    pub fn interface_ids(&self) -> &'a [InterfaceDefinitionId] {
        match self {
            EntityDefinition::Object(item) => &item.as_ref().interface_ids,
            EntityDefinition::Interface(item) => &item.as_ref().interface_ids,
        }
    }

    pub fn interfaces(&self) -> impl Iter<Item = InterfaceDefinition<'a>> + 'a {
        let (schema, interface_ids) = match self {
            EntityDefinition::Object(item) => (item.schema, &item.as_ref().interface_ids),
            EntityDefinition::Interface(item) => (item.schema, &item.as_ref().interface_ids),
        };
        interface_ids.walk(schema)
    }

    pub fn possible_type_ids(&self) -> &[ObjectDefinitionId] {
        match self {
            EntityDefinition::Object(object) => std::array::from_ref(&object.id),
            EntityDefinition::Interface(interface) => &interface.possible_type_ids,
        }
    }
}

impl PartialEq for EntityDefinition<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Interface(l), Self::Interface(r)) => l.id == r.id,
            (Self::Object(l), Self::Object(r)) => l.id == r.id,
            _ => false,
        }
    }
}

impl Eq for EntityDefinition<'_> {}
