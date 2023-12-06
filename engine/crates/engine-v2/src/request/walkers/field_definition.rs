use engine_parser::Pos;
use schema::{FieldId, FieldWalker};

use crate::{
    request::{BoundAnyFieldDefinition, BoundAnyFieldDefinitionId, BoundFieldDefinition},
    response::ResponseKey,
};

use super::{BoundFieldArgumentWalker, OperationWalker};

pub type BoundAnyFieldDefinitionWalker<'a, Extension = ()> =
    OperationWalker<'a, BoundAnyFieldDefinitionId, (), Extension>;
pub type BoundFieldDefinitionWalker<'a, Extension = ()> =
    OperationWalker<'a, &'a BoundFieldDefinition, FieldId, Extension>;

impl<'a, E: Copy> BoundAnyFieldDefinitionWalker<'a, E> {
    pub fn as_field(&self) -> Option<BoundFieldDefinitionWalker<'a, E>> {
        match self.inner() {
            BoundAnyFieldDefinition::TypeName(_) => None,
            BoundAnyFieldDefinition::Field(definition) => Some(self.walk_with(definition, definition.field_id)),
        }
    }

    pub fn name_location(&self) -> Pos {
        match self.inner() {
            BoundAnyFieldDefinition::TypeName(definition) => definition.name_location,
            BoundAnyFieldDefinition::Field(definition) => definition.name_location,
        }
    }
}

impl<'a, E> std::fmt::Debug for BoundAnyFieldDefinitionWalker<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundAnyFieldDefinitionWalker")
            .field("response_key", &&self.operation.response_keys[self.response_key()])
            .field(
                "name",
                &match self.inner() {
                    BoundAnyFieldDefinition::TypeName(_) => "__typename",
                    BoundAnyFieldDefinition::Field(definition) => self.schema.walk(definition.field_id).name(),
                },
            )
            .finish()
    }
}

impl<'a, E> std::ops::Deref for BoundFieldDefinitionWalker<'a, E> {
    type Target = FieldWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema
    }
}

impl<'a, E> BoundFieldDefinitionWalker<'a, E> {
    pub fn response_key(&self) -> ResponseKey {
        self.inner.response_key
    }

    pub fn response_key_str(&self) -> &str {
        &self.operation.response_keys[self.inner.response_key]
    }

    pub fn name_location(&self) -> Pos {
        self.inner.name_location
    }

    pub fn bound_arguments(&self) -> impl ExactSizeIterator<Item = BoundFieldArgumentWalker<'a, E>> + 'a
    where
        E: Copy + 'a,
    {
        let walker = *self;
        self.inner
            .arguments
            .iter()
            .map(move |argument| walker.walk_with(argument, argument.input_value_id))
    }
}

impl<'a, E> std::fmt::Debug for BoundFieldDefinitionWalker<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundFieldDefinition")
            .field("reponse_key", &self.response_key_str())
            .field("field", &self.schema)
            .finish()
    }
}
