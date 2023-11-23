use engine_parser::Pos;
use schema::{FieldWalker, SchemaWalker};

use super::BoundSelectionSetWalker;
use crate::{
    request::{
        BoundAnyFieldDefinition, BoundAnyFieldDefinitionId, BoundField, BoundFieldDefinition, BoundFieldId,
        BoundTypeNameFieldDefinition, Operation,
    },
    response::{BoundResponseKey, ResponseKey},
};

pub struct BoundFieldWalker<'a> {
    pub(in crate::request) schema: SchemaWalker<'a, ()>,
    pub(in crate::request) operation: &'a Operation,
    pub(in crate::request) bound_field: &'a BoundField,
    pub(in crate::request) id: BoundFieldId,
}

impl<'a> BoundFieldWalker<'a> {
    pub fn bound_definition_id(&self) -> BoundAnyFieldDefinitionId {
        self.bound_field.definition_id
    }

    #[allow(dead_code)]
    pub fn bound_field_id(&self) -> BoundFieldId {
        self.id
    }

    pub fn bound_response_key(&self) -> BoundResponseKey {
        self.bound_field.bound_response_key
    }

    pub fn schema_field(&self) -> Option<FieldWalker<'a>> {
        match &self.operation[self.bound_field.definition_id] {
            BoundAnyFieldDefinition::TypeName(_) => None,
            BoundAnyFieldDefinition::Field(definition) => Some(self.schema.walk(definition.field_id)),
        }
    }

    pub fn response_key(&self) -> ResponseKey {
        self.operation[self.bound_field.definition_id].response_key()
    }

    pub fn selection_set(&self) -> Option<BoundSelectionSetWalker<'a>> {
        self.bound_field.selection_set_id.map(|id| BoundSelectionSetWalker {
            schema: self.schema,
            operation: self.operation,
            id,
        })
    }

    pub fn definition(&self) -> BoundAnyFieldDefinitionWalker<'a> {
        self.operation
            .walk_definition(self.schema, self.bound_field.definition_id)
    }
}

pub enum BoundAnyFieldDefinitionWalker<'a> {
    TypeName(&'a BoundTypeNameFieldDefinition),
    Field(BoundFieldDefinitionWalker<'a>),
}

#[derive(Clone)]
pub struct BoundFieldDefinitionWalker<'a> {
    pub(in crate::request) field: FieldWalker<'a>,
    pub(in crate::request) definition: &'a BoundFieldDefinition,
}

impl<'a> BoundAnyFieldDefinitionWalker<'a> {
    pub fn as_field(&self) -> Option<BoundFieldDefinitionWalker<'a>> {
        match self {
            BoundAnyFieldDefinitionWalker::TypeName(_) => None,
            BoundAnyFieldDefinitionWalker::Field(field) => Some(field.clone()),
        }
    }

    pub fn is_typename_meta_field(&self) -> bool {
        match self {
            BoundAnyFieldDefinitionWalker::TypeName(_) => true,
            BoundAnyFieldDefinitionWalker::Field(_) => false,
        }
    }

    pub fn name_location(&self) -> Pos {
        match self {
            BoundAnyFieldDefinitionWalker::TypeName(definition) => definition.name_location,
            BoundAnyFieldDefinitionWalker::Field(field) => field.definition.name_location,
        }
    }
}

impl<'a> std::ops::Deref for BoundFieldDefinitionWalker<'a> {
    type Target = FieldWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.field
    }
}

impl<'a> BoundAnyFieldDefinitionWalker<'a> {}

impl<'a> std::fmt::Debug for BoundFieldWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundFieldWalker")
            .field("response_key", &&self.operation.response_keys[self.response_key()])
            .field("field_name", &self.schema_field().map(|f| f.name()))
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
