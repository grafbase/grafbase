use crate::{
    Definition, EnumId, FieldId, InputObjectId, InputValueId, InterfaceId, ObjectId, ScalarId, Schema, UnionId,
};

/// Small abstraction over the actual names to make easier to deal with
/// renames later.
/// It's only missing enum value names.
pub trait Names: Send + Sync {
    fn field(&self, field_id: FieldId) -> &str;
    fn object(&self, object_id: ObjectId) -> &str;
    fn union(&self, union_id: UnionId) -> &str;
    fn interface(&self, interface_id: InterfaceId) -> &str;
    fn input_value(&self, input_value_id: InputValueId) -> &str;
    fn input_object(&self, input_object_id: InputObjectId) -> &str;
    fn scalar(&self, scalar_id: ScalarId) -> &str;
    fn r#enum(&self, enum_id: EnumId) -> &str;
    fn union_discriminant_key(&self, union_id: UnionId) -> &str;
    fn interface_discriminant_key(&self, interface_id: InterfaceId) -> &str;
    fn conrete_object_id_from_union_discriminant(&self, union_id: UnionId, discriminant: &str) -> Option<ObjectId>;
    fn conrete_object_id_from_interface_discriminant(
        &self,
        interface_id: InterfaceId,
        discriminant: &str,
    ) -> Option<ObjectId>;
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

    fn union_discriminant_key(&self, _union_id: UnionId) -> &str {
        "__typename"
    }

    fn interface_discriminant_key(&self, _interface_id: InterfaceId) -> &str {
        "__typename"
    }

    fn conrete_object_id_from_union_discriminant(&self, union_id: UnionId, discriminant: &str) -> Option<ObjectId> {
        self.definition_by_name(discriminant).and_then(|definition| {
            if let Definition::Object(object_id) = definition {
                if self[union_id].possible_types.contains(&object_id) {
                    Some(object_id)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn conrete_object_id_from_interface_discriminant(
        &self,
        interface_id: InterfaceId,
        discriminant: &str,
    ) -> Option<ObjectId> {
        self.definition_by_name(discriminant).and_then(|definition| {
            if let Definition::Object(object_id) = definition {
                if self[interface_id].possible_types.contains(&object_id) {
                    Some(object_id)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}
