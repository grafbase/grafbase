use crate::{
    Definition, EnumId, FieldId, InputObjectId, InputValueDefinitionId, InterfaceId, ObjectId, ScalarId, Schema,
    UnionId,
};

/// Small abstraction over the actual names to make easier to deal with
/// renames later.
/// It's only missing enum value names.
pub trait Names: Send + Sync {
    fn field<'s>(&self, schema: &'s Schema, field_id: FieldId) -> &'s str {
        &schema[schema[field_id].name]
    }

    fn object<'s>(&self, schema: &'s Schema, object_id: ObjectId) -> &'s str {
        &schema[schema[object_id].name]
    }

    fn union<'s>(&self, schema: &'s Schema, union_id: UnionId) -> &'s str {
        &schema[schema[union_id].name]
    }

    fn interface<'s>(&self, schema: &'s Schema, interface_id: InterfaceId) -> &'s str {
        &schema[schema[interface_id].name]
    }

    fn input_value<'s>(&self, schema: &'s Schema, input_value_id: InputValueDefinitionId) -> &'s str {
        &schema[schema[input_value_id].name]
    }

    fn input_object<'s>(&self, schema: &'s Schema, input_object_id: InputObjectId) -> &'s str {
        &schema[schema[input_object_id].name]
    }

    fn scalar<'s>(&self, schema: &'s Schema, scalar_id: ScalarId) -> &'s str {
        &schema[schema[scalar_id].name]
    }

    fn r#enum<'s>(&self, schema: &'s Schema, enum_id: EnumId) -> &'s str {
        &schema[schema[enum_id].name]
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Used when writing data into the response to determine the actual object id //

    fn union_discriminant_key<'s>(&self, _schema: &'s Schema, _union_id: UnionId) -> &'s str {
        "__typename"
    }

    fn interface_discriminant_key<'s>(&self, _schema: &'s Schema, _interface_id: InterfaceId) -> &'s str {
        "__typename"
    }

    fn concrete_object_id_from_union_discriminant(
        &self,
        schema: &Schema,
        union_id: UnionId,
        discriminant: &str,
    ) -> Option<ObjectId> {
        schema.definition_by_name(discriminant).and_then(|definition| {
            if let Definition::Object(object_id) = definition {
                if schema[union_id].possible_types.contains(&object_id) {
                    Some(object_id)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn concrete_object_id_from_interface_discriminant(
        &self,
        schema: &Schema,
        interface_id: InterfaceId,
        discriminant: &str,
    ) -> Option<ObjectId> {
        schema.definition_by_name(discriminant).and_then(|definition| {
            if let Definition::Object(object_id) = definition {
                if schema[interface_id].possible_types.contains(&object_id) {
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

impl Names for () {}
