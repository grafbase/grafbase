use crate::{FieldId, InterfaceId, ObjectId, Schema, UnionId};

const TYPENAME: &str = "__typename";

pub trait Translator {
    fn field(&self, field_id: FieldId) -> &str;
    fn typename(&self, object_id: ObjectId) -> &str;
    fn union_typename_field(&self, union_id: UnionId) -> &str;
    fn interface_typename_field(&self, interface_id: InterfaceId) -> &str;
}

pub struct SchemaTranslator<'a> {
    schema: &'a Schema,
}

impl<'a> SchemaTranslator<'a> {
    pub fn new(schema: &'a Schema) -> Self {
        Self { schema }
    }
}

impl<'a> Translator for SchemaTranslator<'a> {
    fn field(&self, field_id: FieldId) -> &str {
        &self.schema[self.schema[field_id].name]
    }

    fn typename(&self, object_id: ObjectId) -> &str {
        &self.schema[self.schema[object_id].name]
    }

    fn union_typename_field(&self, _union_id: UnionId) -> &str {
        TYPENAME
    }

    fn interface_typename_field(&self, _interface_id: InterfaceId) -> &str {
        TYPENAME
    }
}
