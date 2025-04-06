use walker::Walk;

use crate::{
    CompositeType, CompositeTypeId, DeprecatedDirective, EntityDefinition, EntityDefinitionId, TypeDefinition,
    TypeDefinitionId, TypeSystemDirective, TypeSystemDirectiveId,
};

impl<'a> TypeDefinition<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            TypeDefinition::Enum(item) => item.name(),
            TypeDefinition::InputObject(item) => item.name(),
            TypeDefinition::Interface(item) => item.name(),
            TypeDefinition::Object(item) => item.name(),
            TypeDefinition::Scalar(item) => item.name(),
            TypeDefinition::Union(item) => item.name(),
        }
    }

    pub fn description(&self) -> Option<&'a str> {
        match self {
            TypeDefinition::Enum(item) => item.description(),
            TypeDefinition::InputObject(item) => item.description(),
            TypeDefinition::Interface(item) => item.description(),
            TypeDefinition::Object(item) => item.description(),
            TypeDefinition::Scalar(item) => item.description(),
            TypeDefinition::Union(item) => item.description(),
        }
    }

    pub fn directive_ids(&self) -> &'a [TypeSystemDirectiveId] {
        match self {
            TypeDefinition::Enum(item) => &item.as_ref().directive_ids,
            TypeDefinition::InputObject(item) => &item.as_ref().directive_ids,
            TypeDefinition::Interface(item) => &item.as_ref().directive_ids,
            TypeDefinition::Object(item) => &item.as_ref().directive_ids,
            TypeDefinition::Scalar(item) => &item.as_ref().directive_ids,
            TypeDefinition::Union(item) => &item.as_ref().directive_ids,
        }
    }

    pub fn directives(&self) -> impl Iterator<Item = TypeSystemDirective<'a>> + 'a {
        let (schema, directive_ids) = match self {
            TypeDefinition::Enum(item) => (item.schema, &item.as_ref().directive_ids),
            TypeDefinition::InputObject(item) => (item.schema, &item.as_ref().directive_ids),
            TypeDefinition::Interface(item) => (item.schema, &item.as_ref().directive_ids),
            TypeDefinition::Object(item) => (item.schema, &item.as_ref().directive_ids),
            TypeDefinition::Scalar(item) => (item.schema, &item.as_ref().directive_ids),
            TypeDefinition::Union(item) => (item.schema, &item.as_ref().directive_ids),
        };
        directive_ids.walk(schema)
    }

    pub fn as_entity(&self) -> Option<EntityDefinition<'a>> {
        match self {
            TypeDefinition::Object(object) => Some(EntityDefinition::Object(*object)),
            TypeDefinition::Interface(interface) => Some(EntityDefinition::Interface(*interface)),
            _ => None,
        }
    }

    pub fn as_composite_type(&self) -> Option<CompositeType<'a>> {
        match self {
            TypeDefinition::Object(object) => Some(CompositeType::Object(*object)),
            TypeDefinition::Interface(interface) => Some(CompositeType::Interface(*interface)),
            TypeDefinition::Union(union) => Some(CompositeType::Union(*union)),
            _ => None,
        }
    }

    pub fn is_entity(&self) -> bool {
        matches!(self, TypeDefinition::Object(_) | TypeDefinition::Interface(_))
    }

    pub fn is_composite_type(&self) -> bool {
        matches!(
            self,
            TypeDefinition::Object(_) | TypeDefinition::Interface(_) | TypeDefinition::Union(_)
        )
    }

    pub fn cost(&self) -> Option<i32> {
        self.directives().find_map(|directive| match directive {
            TypeSystemDirective::Cost(cost) => Some(cost.weight),
            _ => None,
        })
    }

    pub fn is_inaccessible(&self) -> bool {
        match self {
            TypeDefinition::Enum(enm) => enm.is_inaccessible(),
            TypeDefinition::InputObject(input_object) => input_object.is_inaccessible(),
            TypeDefinition::Interface(interface) => interface.is_inaccessible(),
            TypeDefinition::Object(object) => object.is_inaccessible(),
            TypeDefinition::Scalar(scalar) => scalar.is_inaccessible(),
            TypeDefinition::Union(union) => union.is_inaccessible(),
        }
    }

    pub fn has_deprecated(&self) -> Option<DeprecatedDirective<'_>> {
        self.directives().find_map(|directive| directive.as_deprecated())
    }
}

impl TypeDefinitionId {
    pub fn as_entity(&self) -> Option<EntityDefinitionId> {
        match self {
            TypeDefinitionId::Object(object) => Some(EntityDefinitionId::Object(*object)),
            TypeDefinitionId::Interface(interface) => Some(EntityDefinitionId::Interface(*interface)),
            _ => None,
        }
    }

    pub fn as_composite_type(&self) -> Option<CompositeTypeId> {
        match self {
            TypeDefinitionId::Object(object) => Some(CompositeTypeId::Object(*object)),
            TypeDefinitionId::Interface(interface) => Some(CompositeTypeId::Interface(*interface)),
            TypeDefinitionId::Union(union) => Some(CompositeTypeId::Union(*union)),
            _ => None,
        }
    }

    pub fn is_entity(&self) -> bool {
        matches!(self, TypeDefinitionId::Object(_) | TypeDefinitionId::Interface(_))
    }

    pub fn is_composite_type(&self) -> bool {
        matches!(
            self,
            TypeDefinitionId::Object(_) | TypeDefinitionId::Interface(_) | TypeDefinitionId::Union(_)
        )
    }
}

impl From<EntityDefinitionId> for TypeDefinitionId {
    fn from(id: EntityDefinitionId) -> Self {
        match id {
            EntityDefinitionId::Object(id) => TypeDefinitionId::Object(id),
            EntityDefinitionId::Interface(id) => TypeDefinitionId::Interface(id),
        }
    }
}

impl From<CompositeTypeId> for TypeDefinitionId {
    fn from(id: CompositeTypeId) -> Self {
        match id {
            CompositeTypeId::Object(id) => TypeDefinitionId::Object(id),
            CompositeTypeId::Interface(id) => TypeDefinitionId::Interface(id),
            CompositeTypeId::Union(id) => TypeDefinitionId::Union(id),
        }
    }
}
