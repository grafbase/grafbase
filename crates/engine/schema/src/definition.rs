use walker::Walk;

use crate::{
    CompositeType, CompositeTypeId, Definition, DefinitionId, EntityDefinition, EntityDefinitionId, TypeSystemDirective,
};

impl<'a> Definition<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            Definition::Enum(item) => item.name(),
            Definition::InputObject(item) => item.name(),
            Definition::Interface(item) => item.name(),
            Definition::Object(item) => item.name(),
            Definition::Scalar(item) => item.name(),
            Definition::Union(item) => item.name(),
        }
    }

    pub fn directives(&self) -> impl Iterator<Item = TypeSystemDirective<'a>> + 'a {
        let (schema, directive_ids) = match self {
            Definition::Enum(item) => (item.schema, &item.as_ref().directive_ids),
            Definition::InputObject(item) => (item.schema, &item.as_ref().directive_ids),
            Definition::Interface(item) => (item.schema, &item.as_ref().directive_ids),
            Definition::Object(item) => (item.schema, &item.as_ref().directive_ids),
            Definition::Scalar(item) => (item.schema, &item.as_ref().directive_ids),
            Definition::Union(item) => (item.schema, &item.as_ref().directive_ids),
        };
        directive_ids.walk(schema)
    }

    pub fn as_entity(&self) -> Option<EntityDefinition<'a>> {
        match self {
            Definition::Object(object) => Some(EntityDefinition::Object(*object)),
            Definition::Interface(interface) => Some(EntityDefinition::Interface(*interface)),
            _ => None,
        }
    }

    pub fn as_composite_type(&self) -> Option<CompositeType<'a>> {
        match self {
            Definition::Object(object) => Some(CompositeType::Object(*object)),
            Definition::Interface(interface) => Some(CompositeType::Interface(*interface)),
            Definition::Union(union) => Some(CompositeType::Union(*union)),
            _ => None,
        }
    }

    pub fn is_entity(&self) -> bool {
        matches!(self, Definition::Object(_) | Definition::Interface(_))
    }

    pub fn is_composite_type(&self) -> bool {
        matches!(
            self,
            Definition::Object(_) | Definition::Interface(_) | Definition::Union(_)
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
            Definition::Enum(enm) => enm.is_inaccessible(),
            Definition::InputObject(input_object) => input_object.is_inaccessible(),
            Definition::Interface(interface) => interface.is_inaccessible(),
            Definition::Object(object) => object.is_inaccessible(),
            Definition::Scalar(scalar) => scalar.is_inaccessible(),
            Definition::Union(union) => union.is_inaccessible(),
        }
    }
}

impl DefinitionId {
    pub fn as_entity(&self) -> Option<EntityDefinitionId> {
        match self {
            DefinitionId::Object(object) => Some(EntityDefinitionId::Object(*object)),
            DefinitionId::Interface(interface) => Some(EntityDefinitionId::Interface(*interface)),
            _ => None,
        }
    }

    pub fn as_composite_type(&self) -> Option<CompositeTypeId> {
        match self {
            DefinitionId::Object(object) => Some(CompositeTypeId::Object(*object)),
            DefinitionId::Interface(interface) => Some(CompositeTypeId::Interface(*interface)),
            DefinitionId::Union(union) => Some(CompositeTypeId::Union(*union)),
            _ => None,
        }
    }

    pub fn is_entity(&self) -> bool {
        matches!(self, DefinitionId::Object(_) | DefinitionId::Interface(_))
    }

    pub fn is_composite_type(&self) -> bool {
        matches!(
            self,
            DefinitionId::Object(_) | DefinitionId::Interface(_) | DefinitionId::Union(_)
        )
    }
}

impl From<EntityDefinitionId> for DefinitionId {
    fn from(id: EntityDefinitionId) -> Self {
        match id {
            EntityDefinitionId::Object(id) => DefinitionId::Object(id),
            EntityDefinitionId::Interface(id) => DefinitionId::Interface(id),
        }
    }
}

impl From<CompositeTypeId> for DefinitionId {
    fn from(id: CompositeTypeId) -> Self {
        match id {
            CompositeTypeId::Object(id) => DefinitionId::Object(id),
            CompositeTypeId::Interface(id) => DefinitionId::Interface(id),
            CompositeTypeId::Union(id) => DefinitionId::Union(id),
        }
    }
}
