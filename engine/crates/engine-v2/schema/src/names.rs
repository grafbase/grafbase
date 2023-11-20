use crate::{EnumId, FieldId, InputObjectId, InputValueId, InterfaceId, ObjectId, ScalarId, Schema, UnionId};

/// Small abstraction over the actual names to make easier to deal with
/// renames later.
/// It's only missing enum value names.
pub trait Names {
    fn field(&self, field_id: FieldId) -> &str;
    fn object(&self, object_id: ObjectId) -> &str;
    fn union(&self, union_id: UnionId) -> &str;
    fn interface(&self, interface_id: InterfaceId) -> &str;
    fn input_value(&self, input_value_id: InputValueId) -> &str;
    fn input_object(&self, input_object_id: InputObjectId) -> &str;
    fn scalar(&self, scalar_id: ScalarId) -> &str;
    fn r#enum(&self, enum_id: EnumId) -> &str;
}

impl Names for Schema {
    fn field(&self, field_id: FieldId) -> &str {
        &self[self[field_id].name]
    }

    fn object(&self, object_id: ObjectId) -> &str {
        &self[self[object_id].name]
    }

    fn union(&self, union_id: UnionId) -> &str {
        &self[self[union_id].name]
    }

    fn interface(&self, interface_id: InterfaceId) -> &str {
        &self[self[interface_id].name]
    }

    fn input_value(&self, input_value_id: InputValueId) -> &str {
        &self[self[input_value_id].name]
    }

    fn input_object(&self, input_object_id: InputObjectId) -> &str {
        &self[self[input_object_id].name]
    }

    fn scalar(&self, scalar_id: ScalarId) -> &str {
        &self[self[scalar_id].name]
    }

    fn r#enum(&self, enum_id: EnumId) -> &str {
        &self[self[enum_id].name]
    }
}
