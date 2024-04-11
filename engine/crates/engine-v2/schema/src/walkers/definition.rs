use super::{field::FieldDefinitionWalker, SchemaWalker};
use crate::{
    Definition, EnumWalker, InputObjectWalker, InterfaceWalker, ObjectWalker, ScalarType, ScalarWalker, StringId,
    TypeSystemDirectivesWalker,
};

pub type DefinitionWalker<'a> = SchemaWalker<'a, Definition>;

impl<'a> DefinitionWalker<'a> {
    pub fn id(&self) -> Definition {
        self.item
    }

    pub fn name(&self) -> &'a str {
        match self.item {
            Definition::Scalar(s) => self.names.scalar(self.schema, s),
            Definition::Object(o) => self.names.object(self.schema, o),
            Definition::Interface(i) => self.names.interface(self.schema, i),
            Definition::Union(u) => self.names.union(self.schema, u),
            Definition::Enum(e) => self.names.r#enum(self.schema, e),
            Definition::InputObject(io) => self.names.input_object(self.schema, io),
        }
    }

    pub fn schema_name_id(&self) -> StringId {
        match self.item {
            Definition::Scalar(s) => self.schema[s].name,
            Definition::Object(o) => self.schema[o].name,
            Definition::Interface(i) => self.schema[i].name,
            Definition::Union(u) => self.schema[u].name,
            Definition::Enum(e) => self.schema[e].name,
            Definition::InputObject(io) => self.schema[io].name,
        }
    }

    pub fn schema_description_id(&self) -> Option<StringId> {
        match self.item {
            Definition::Scalar(s) => self.schema[s].description,
            Definition::Object(o) => self.schema[o].description,
            Definition::Interface(i) => self.schema[i].description,
            Definition::Union(u) => self.schema[u].description,
            Definition::Enum(e) => self.schema[e].description,
            Definition::InputObject(io) => self.schema[io].description,
        }
    }

    pub fn fields(&self) -> Option<Box<dyn Iterator<Item = FieldDefinitionWalker<'a>> + 'a>> {
        match self.item {
            Definition::Object(o) => Some(Box::new(self.walk(o).fields())),
            Definition::Interface(i) => Some(Box::new(self.walk(i).fields())),
            _ => None,
        }
    }

    pub fn interfaces(&self) -> Option<Box<dyn ExactSizeIterator<Item = InterfaceWalker<'a>> + 'a>> {
        match self.item {
            Definition::Object(o) => Some(Box::new(self.walk(o).interfaces())),
            Definition::Interface(i) => Some(Box::new(self.walk(i).interfaces())),
            _ => None,
        }
    }

    pub fn possible_types(&self) -> Option<Box<dyn ExactSizeIterator<Item = ObjectWalker<'a>> + 'a>> {
        match self.item {
            Definition::Interface(i) => Some(Box::new(self.walk(i).possible_types())),
            Definition::Union(u) => Some(Box::new(self.walk(u).possible_types())),
            _ => None,
        }
    }

    pub fn as_enum(&self) -> Option<EnumWalker<'a>> {
        match self.item {
            Definition::Enum(e) => Some(self.walk(e)),
            _ => None,
        }
    }

    pub fn as_input_object(&self) -> Option<InputObjectWalker<'a>> {
        match self.item {
            Definition::InputObject(io) => Some(self.walk(io)),
            _ => None,
        }
    }

    pub fn as_scalar(&self) -> Option<ScalarWalker<'a>> {
        match self.item {
            Definition::Scalar(s) => Some(self.walk(s)),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<ObjectWalker<'a>> {
        match self.item {
            Definition::Object(s) => Some(self.walk(s)),
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

impl<'a> From<ObjectWalker<'a>> for DefinitionWalker<'a> {
    fn from(value: ObjectWalker<'a>) -> Self {
        value.walk(value.item.into())
    }
}

impl<'a> From<InterfaceWalker<'a>> for DefinitionWalker<'a> {
    fn from(value: InterfaceWalker<'a>) -> Self {
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
