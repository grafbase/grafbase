use engine_parser::Pos;
use schema::{FieldId, FieldWalker};

use crate::{
    plan::ExtraField,
    request::{BoundAnyFieldDefinition, BoundAnyFieldDefinitionId, BoundFieldDefinition},
    response::ResponseKey,
};

use super::{BoundFieldArgumentWalker, OperationWalker, PlanExt};

pub type BoundAnyFieldDefinitionWalker<'a, Extension = ()> =
    OperationWalker<'a, BoundAnyFieldDefinitionId, (), Extension>;
pub type BoundFieldDefinitionWalker<'a, Extension = ()> =
    OperationWalker<'a, &'a BoundFieldDefinition, FieldId, Extension>;

impl<'a, E: Copy> BoundAnyFieldDefinitionWalker<'a, E> {
    pub fn as_field(&self) -> Option<BoundFieldDefinitionWalker<'a, E>> {
        match self.get() {
            BoundAnyFieldDefinition::TypeName(_) => None,
            BoundAnyFieldDefinition::Field(definition) => Some(self.walk_with(definition, definition.field_id)),
        }
    }

    pub fn name_location(&self) -> Pos {
        match self.get() {
            BoundAnyFieldDefinition::TypeName(definition) => definition.name_location,
            BoundAnyFieldDefinition::Field(definition) => definition.name_location,
        }
    }

    pub fn response_key_str(&self) -> &'a str {
        &self.operation.response_keys[self.get().response_key()]
    }
}

impl<'a, E: Copy> std::fmt::Debug for BoundAnyFieldDefinitionWalker<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("BoundAnyFieldDefinition");
        let name = match self.get() {
            BoundAnyFieldDefinition::TypeName(_) => "__typename",
            BoundAnyFieldDefinition::Field(definition) => self.schema_walker.walk(definition.field_id).name(),
        };
        if self.response_key_str() != name {
            fmt.field("key", &self.response_key_str());
        }
        fmt.field("name", &name).finish()
    }
}

impl<'a, E> std::ops::Deref for BoundFieldDefinitionWalker<'a, E> {
    type Target = FieldWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}

impl<'a, E> BoundFieldDefinitionWalker<'a, E> {
    pub fn response_key(&self) -> ResponseKey {
        self.wrapped.response_key
    }

    pub fn response_key_str(&self) -> &str {
        &self.operation.response_keys[self.wrapped.response_key]
    }

    pub fn bound_arguments(&self) -> impl ExactSizeIterator<Item = BoundFieldArgumentWalker<'a, E>> + 'a
    where
        E: Copy + 'a,
    {
        let walker = *self;
        self.wrapped
            .arguments
            .iter()
            .map(move |argument| walker.walk_with(argument, argument.input_value_id))
    }
}

impl<'a, E> std::fmt::Debug for BoundFieldDefinitionWalker<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("BoundFieldDefinition");
        if self.response_key_str() != self.name() {
            fmt.field("key", &self.response_key_str());
        }
        fmt.field("field", &self.schema_walker).finish()
    }
}

#[derive(Clone, Copy)]
pub enum PlanFieldDefinition<'a> {
    Query(BoundFieldDefinitionWalker<'a, PlanExt<'a>>),
    Extra {
        schema_field: FieldWalker<'a>,
        extra: &'a ExtraField,
    },
}

impl<'a> PlanFieldDefinition<'a> {
    pub fn name_location(&self) -> Option<Pos> {
        match self {
            PlanFieldDefinition::Query(walker) => Some(walker.wrapped.name_location),
            PlanFieldDefinition::Extra { .. } => None,
        }
    }
    pub fn bound_arguments(&self) -> impl ExactSizeIterator<Item = BoundFieldArgumentWalker<'a, PlanExt<'a>>> + 'a {
        let arguments = match self {
            PlanFieldDefinition::Query(walker) => walker.bound_arguments().collect(),
            PlanFieldDefinition::Extra { .. } => vec![],
        };
        arguments.into_iter()
    }
}

impl<'a> std::ops::Deref for PlanFieldDefinition<'a> {
    type Target = FieldWalker<'a>;

    fn deref(&self) -> &Self::Target {
        match self {
            PlanFieldDefinition::Query(walker) => walker,
            PlanFieldDefinition::Extra { schema_field, .. } => schema_field,
        }
    }
}
