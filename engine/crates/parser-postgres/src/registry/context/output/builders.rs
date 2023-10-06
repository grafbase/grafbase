use engine::registry::{Constraint, EnumType, InputObjectType, MetaEnumValue, MetaField, MetaInputValue, ObjectType};
use postgres_types::database_definition::{RelationId, TableColumnId, TableId, UniqueConstraintId};

#[derive(Debug)]
pub struct ObjectTypeBuilder {
    pub(super) object_type: ObjectType,
    pub(super) type_mapping: Vec<(String, TableId)>,
    pub(super) field_mapping: Vec<(String, TableColumnId)>,
    pub(super) relation_mapping: Vec<(String, RelationId)>,
}

impl ObjectTypeBuilder {
    pub(super) fn new(name: impl Into<String>, table_id: TableId) -> Self {
        let type_name = name.into();

        Self {
            object_type: ObjectType::new(&type_name, []),
            type_mapping: vec![(type_name, table_id)],
            field_mapping: Vec::new(),
            relation_mapping: Vec::new(),
        }
    }

    pub(crate) fn push_scalar_field(&mut self, field: MetaField, column_id: TableColumnId) {
        self.field_mapping.push((field.name.to_string(), column_id));
        self.push_non_mapped_scalar_field(field);
    }

    pub(crate) fn push_non_mapped_scalar_field(&mut self, field: MetaField) {
        self.object_type.fields.insert(field.name.to_string(), field);
    }

    pub(crate) fn push_relation_field(&mut self, field: MetaField, id: RelationId) {
        self.relation_mapping.push((field.name.clone(), id));
        self.object_type.fields.insert(field.name.to_string(), field);
    }

    pub(crate) fn push_constraint(&mut self, constraint: Constraint) {
        self.object_type.constraints.push(constraint);
    }
}

#[derive(Debug)]
pub struct InputTypeBuilder {
    pub(super) input_object_type: InputObjectType,
    pub(super) type_mapping: Vec<(String, TableId)>,
    pub(super) field_mapping: Vec<(String, TableColumnId)>,
    pub(super) unique_constraint_mapping: Vec<(String, UniqueConstraintId)>,
    pub(super) relation_mapping: Vec<(String, RelationId)>,
    pub(super) nested: Vec<InputObjectType>,
}

impl InputTypeBuilder {
    pub(super) fn new(name: impl Into<String>, table_id: TableId) -> Self {
        let name = name.into();

        Self {
            input_object_type: InputObjectType::new(name.clone(), []),
            type_mapping: vec![(name.clone(), table_id)],
            field_mapping: Vec::new(),
            unique_constraint_mapping: Vec::new(),
            relation_mapping: Vec::new(),
            nested: Vec::new(),
        }
    }

    pub(crate) fn map_unique_constraint(&mut self, field: &str, constraint_id: UniqueConstraintId) {
        self.unique_constraint_mapping.push((field.to_string(), constraint_id));
    }

    pub(crate) fn push_input_column(&mut self, value: MetaInputValue, column_id: TableColumnId) {
        self.field_mapping.push((value.name.to_string(), column_id));
        self.push_input_value(value);
    }

    pub(crate) fn push_input_value(&mut self, value: MetaInputValue) {
        self.input_object_type
            .input_fields
            .insert(value.name.to_string(), value);
    }

    pub(crate) fn push_input_relation(&mut self, value: MetaInputValue, id: RelationId) {
        self.relation_mapping.push((value.name.clone(), id));
        self.push_input_value(value);
    }

    pub(crate) fn oneof(&mut self, value: bool) {
        self.input_object_type.oneof = value;
    }

    pub fn with_input_type<F>(&mut self, name: &str, table_id: TableId, f: F)
    where
        F: FnOnce(&mut InputTypeBuilder),
    {
        let mut builder = InputTypeBuilder::new(name, table_id);

        f(&mut builder);

        self.type_mapping.extend(builder.type_mapping);
        self.field_mapping.extend(builder.field_mapping);
        self.unique_constraint_mapping.extend(builder.unique_constraint_mapping);
        self.nested.push(builder.input_object_type);
    }
}

pub struct EnumBuilder {
    pub(super) enum_type: EnumType,
}

impl EnumBuilder {
    pub(super) fn new(name: impl Into<String>) -> Self {
        Self {
            enum_type: EnumType::new(name.into(), []),
        }
    }

    pub fn push_variant(&mut self, variant: MetaEnumValue) {
        self.enum_type.enum_values.insert(variant.name.clone(), variant);
    }
}
