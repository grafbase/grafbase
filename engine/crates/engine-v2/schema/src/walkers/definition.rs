use super::{field::FieldDefinition, SchemaWalker};
use crate::{
    Definition, EntityDefinitionId, EntityWalker, EnumDefinitionWalker, InputObjectDefinition, InterfaceDefinition,
    ObjectDefinition, ScalarDefinition, ScalarType, StringId, TypeSystemDirectivesWalker,
};

pub type DefinitionWalker<'a> = SchemaWalker<'a, Definition>;

impl<'a> DefinitionWalker<'a> {
    pub fn id(&self) -> Definition {
        self.item
    }

    pub fn name(&self) -> &'a str {
        &self.schema[self.schema_name_id()]
    }

    pub fn schema_name_id(&self) -> StringId {
        match self.item {
            Definition::Scalar(s) => self.schema[s].name_id,
            Definition::Object(o) => self.schema[o].name_id,
            Definition::Interface(i) => self.schema[i].name_id,
            Definition::Union(u) => self.schema[u].name_id,
            Definition::Enum(e) => self.schema[e].name_id,
            Definition::InputObject(io) => self.schema[io].name_id,
        }
    }

    pub fn schema_description_id(&self) -> Option<StringId> {
        match self.item {
            Definition::Scalar(s) => self.schema[s].description_id,
            Definition::Object(o) => self.schema[o].description_id,
            Definition::Interface(i) => self.schema[i].description_id,
            Definition::Union(u) => self.schema[u].description_id,
            Definition::Enum(e) => self.schema[e].description_id,
            Definition::InputObject(io) => self.schema[io].description_id,
        }
    }

    pub fn fields(&self) -> Option<Box<dyn Iterator<Item = FieldDefinition<'a>> + 'a>> {
        match self.item {
            Definition::Object(o) => Some(Box::new(self.walk(o).fields())),
            Definition::Interface(i) => Some(Box::new(self.walk(i).fields())),
            _ => None,
        }
    }

    pub fn interfaces(&self) -> Option<Box<dyn ExactSizeIterator<Item = InterfaceDefinition<'a>> + 'a>> {
        match self.item {
            Definition::Object(o) => Some(Box::new(self.walk(o).interfaces())),
            Definition::Interface(i) => Some(Box::new(self.walk(i).interfaces())),
            _ => None,
        }
    }

    pub fn possible_types(&self) -> Option<Box<dyn ExactSizeIterator<Item = ObjectDefinition<'a>> + 'a>> {
        match self.item {
            Definition::Interface(i) => Some(Box::new(self.walk(i).possible_types())),
            Definition::Union(u) => Some(Box::new(self.walk(u).possible_types())),
            _ => None,
        }
    }

    pub fn as_enum(&self) -> Option<EnumDefinitionWalker<'a>> {
        match self.item {
            Definition::Enum(e) => Some(self.walk(e)),
            _ => None,
        }
    }

    pub fn as_input_object(&self) -> Option<InputObjectDefinition<'a>> {
        match self.item {
            Definition::InputObject(io) => Some(self.walk(io)),
            _ => None,
        }
    }

    pub fn as_scalar(&self) -> Option<ScalarDefinition<'a>> {
        match self.item {
            Definition::Scalar(s) => Some(self.walk(s)),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<ObjectDefinition<'a>> {
        match self.item {
            Definition::Object(s) => Some(self.walk(s)),
            _ => None,
        }
    }

    pub fn as_entity(&self) -> Option<EntityWalker<'a>> {
        match self.item {
            Definition::Object(id) => Some(self.walk(EntityDefinitionId::Object(id))),
            Definition::Interface(s) => Some(self.walk(EntityDefinitionId::Interface(s))),
            _ => None,
        }
    }

    pub fn scalar_type(&self) -> Option<ScalarType> {
        match self.item {
            Definition::Scalar(id) => Some(self.schema[id].ty),
            Definition::Enum(_) => Some(ScalarType::String),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self.item, Definition::Object(_))
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        match self.item {
            Definition::Scalar(s) => self.walk(s).directives(),
            Definition::Object(o) => self.walk(o).directives(),
            Definition::Interface(i) => self.walk(i).directives(),
            Definition::Union(u) => self.walk(u).directives(),
            Definition::Enum(e) => self.walk(e).directives(),
            Definition::InputObject(io) => self.walk(io).directives(),
        }
    }
}

impl<'a> From<ObjectDefinition<'a>> for DefinitionWalker<'a> {
    fn from(value: ObjectDefinition<'a>) -> Self {
        value.walk(value.item.into())
    }
}

impl<'a> From<InterfaceDefinition<'a>> for DefinitionWalker<'a> {
    fn from(value: InterfaceDefinition<'a>) -> Self {
        value.walk(value.item.into())
    }
}

impl<'a> std::fmt::Debug for DefinitionWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Definition");
        match self.item {
            Definition::Scalar(s) => debug.field("inner", &self.walk(s)),
            Definition::Object(o) => debug.field("inner", &self.walk(o)),
            Definition::Interface(i) => debug.field("inner", &self.walk(i)),
            Definition::Union(u) => debug.field("inner", &self.walk(u)),
            Definition::Enum(e) => debug.field("inner", &self.walk(e)),
            Definition::InputObject(io) => debug.field("inner", &self.walk(io)),
        };
        debug.finish()
    }
}
