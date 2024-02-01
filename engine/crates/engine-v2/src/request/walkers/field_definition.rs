use schema::{FieldId, FieldWalker};

use crate::{
    request::{BoundAnyFieldDefinition, BoundAnyFieldDefinitionId, BoundFieldDefinition, Location},
    response::ResponseKey,
};

use super::{BoundFieldArgumentWalker, OperationWalker};

pub type BoundAnyFieldDefinitionWalker<'a, CtxOrUnit = ()> =
    OperationWalker<'a, BoundAnyFieldDefinitionId, (), CtxOrUnit>;
pub type BoundFieldDefinitionWalker<'a, CtxOrUnit = ()> =
    OperationWalker<'a, &'a BoundFieldDefinition, FieldId, CtxOrUnit>;

impl<'a, C: Copy> BoundAnyFieldDefinitionWalker<'a, C> {
    pub fn as_field(&self) -> Option<BoundFieldDefinitionWalker<'a, C>> {
        match self.as_ref() {
            BoundAnyFieldDefinition::TypeName(_) => None,
            BoundAnyFieldDefinition::Field(definition) => Some(self.walk_with(definition, definition.field_id)),
        }
    }

    pub fn schema_name(&self) -> &'a str {
        match self.as_ref() {
            BoundAnyFieldDefinition::TypeName(_) => "__typename",
            BoundAnyFieldDefinition::Field(definition) => self.schema_walker.walk(definition.field_id).name(),
        }
    }

    pub fn name_location(&self) -> Location {
        match self.as_ref() {
            BoundAnyFieldDefinition::TypeName(definition) => definition.name_location,
            BoundAnyFieldDefinition::Field(definition) => definition.name_location,
        }
    }

    pub fn response_key_str(&self) -> &'a str {
        &self.operation.response_keys[self.as_ref().response_key()]
    }
}

impl<'a, C: Copy> std::fmt::Debug for BoundAnyFieldDefinitionWalker<'a, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("BoundAnyFieldDefinition");
        let name = match self.as_ref() {
            BoundAnyFieldDefinition::TypeName(_) => "__typename",
            BoundAnyFieldDefinition::Field(definition) => self.schema_walker.walk(definition.field_id).name(),
        };
        if self.response_key_str() != name {
            fmt.field("key", &self.response_key_str());
        }
        fmt.field("name", &name).finish()
    }
}

impl<'a, C> std::ops::Deref for BoundFieldDefinitionWalker<'a, C> {
    type Target = FieldWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}

impl<'a, C> BoundFieldDefinitionWalker<'a, C> {
    pub fn response_key(&self) -> ResponseKey {
        self.item.response_key
    }

    pub fn response_key_str(&self) -> &str {
        &self.operation.response_keys[self.item.response_key]
    }

    pub fn bound_arguments(&self) -> impl ExactSizeIterator<Item = BoundFieldArgumentWalker<'a, C>> + 'a
    where
        C: Copy + 'a,
    {
        let walker = *self;
        self.item
            .arguments
            .iter()
            .map(move |argument| walker.walk_with(argument, argument.input_value_id))
    }
}

impl<'a, C> std::fmt::Debug for BoundFieldDefinitionWalker<'a, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("BoundFieldDefinition");
        if self.response_key_str() != self.name() {
            fmt.field("key", &self.response_key_str());
        }
        fmt.field("field", &self.schema_walker).finish()
    }
}
