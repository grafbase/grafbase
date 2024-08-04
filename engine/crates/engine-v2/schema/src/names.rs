use crate::{
    Definition, EnumDefinitionId, EnumValueId, FieldDefinitionId, InputObjectDefinitionId, InputValueDefinitionId,
    InterfaceDefinitionId, ObjectDefinitionId, ScalarDefinitionId, Schema, UnionDefinitionId,
};

/// Small abstraction over the actual names to make easier to deal with
/// renames later.
/// It's only missing enum value names.
pub trait Names: Send + Sync {
    fn field<'s>(&self, schema: &'s Schema, field_id: FieldDefinitionId) -> &'s str {
        &schema[schema[field_id].name]
    }

    fn object<'s>(&self, schema: &'s Schema, object_id: ObjectDefinitionId) -> &'s str {
        &schema[schema[object_id].name]
    }

    fn union<'s>(&self, schema: &'s Schema, union_id: UnionDefinitionId) -> &'s str {
        &schema[schema[union_id].name]
    }

    fn interface<'s>(&self, schema: &'s Schema, interface_id: InterfaceDefinitionId) -> &'s str {
        &schema[schema[interface_id].name]
    }

    fn input_value<'s>(&self, schema: &'s Schema, input_value_id: InputValueDefinitionId) -> &'s str {
        &schema[schema[input_value_id].name]
    }

    fn input_object<'s>(&self, schema: &'s Schema, input_object_id: InputObjectDefinitionId) -> &'s str {
        &schema[schema[input_object_id].name]
    }

    fn scalar<'s>(&self, schema: &'s Schema, scalar_id: ScalarDefinitionId) -> &'s str {
        &schema[schema[scalar_id].name]
    }

    fn r#enum<'s>(&self, schema: &'s Schema, enum_id: EnumDefinitionId) -> &'s str {
        &schema[schema[enum_id].name]
    }

    fn enum_value<'s>(&self, schema: &'s Schema, enum_value_id: EnumValueId) -> &'s str {
        &schema[schema[enum_value_id].name]
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Used when writing data into the response to determine the actual object id //

    fn union_discriminant_key<'s>(&self, _schema: &'s Schema, _union_id: UnionDefinitionId) -> &'s str {
        "__typename"
    }

    fn interface_discriminant_key<'s>(&self, _schema: &'s Schema, _interface_id: InterfaceDefinitionId) -> &'s str {
        "__typename"
    }

    fn concrete_object_id_from_union_discriminant(
        &self,
        schema: &Schema,
        union_id: UnionDefinitionId,
        discriminant: &str,
    ) -> Option<ObjectDefinitionId> {
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
        interface_id: InterfaceDefinitionId,
        discriminant: &str,
    ) -> Option<ObjectDefinitionId> {
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
