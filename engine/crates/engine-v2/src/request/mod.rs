mod bind;
// mod flat_selection_set;
mod ids;
mod parse;
mod path;
mod selection_set;
mod variable;
mod walkers;

pub use bind::{BindError, BindResult};
pub use engine_parser::{types::OperationType, Pos};
// pub use flat_selection_set::*;
pub use ids::*;
pub use parse::{parse_operation, ParseError, UnboundOperation};
pub use path::{FlatTypeCondition, QueryPath, QueryPathSegment};
use schema::{ObjectId, Schema, SchemaWalker};
pub use selection_set::*;
pub use variable::VariableDefinition;
pub use walkers::*;

use crate::response::ResponseKeys;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableId(usize);

pub struct Operation {
    pub ty: OperationType,
    pub root_object_id: ObjectId,
    pub name: Option<String>,
    pub root_selection_set_id: BoundSelectionSetId,
    pub selection_sets: Vec<BoundSelectionSet>,
    pub fields: Vec<BoundField>,
    pub response_keys: ResponseKeys,
    pub fragment_definitions: Vec<BoundFragmentDefinition>,
    pub field_definitions: Vec<BoundAnyFieldDefinition>,
    pub variable_definitions: Vec<VariableDefinition>,
}

impl Operation {
    /// Binds an unbound operation to a schema. All field names are mapped to their actual field
    /// id in the schema. At this stage the operation might not be resolvable but it should make
    /// sense given the schema types.
    pub fn bind(schema: &Schema, unbound_operation: UnboundOperation) -> BindResult<Self> {
        bind::bind(schema, unbound_operation)
    }

    pub fn walk_selection_set<'a>(
        &'a self,
        schema: SchemaWalker<'a, ()>,
        selection_set_id: BoundSelectionSetId,
    ) -> BoundSelectionSetWalker<'a> {
        BoundSelectionSetWalker {
            schema,
            operation: self,
            id: selection_set_id,
        }
    }

    pub fn walk_field<'a>(&'a self, schema: SchemaWalker<'a, ()>, id: BoundFieldId) -> BoundFieldWalker<'a> {
        BoundFieldWalker {
            schema,
            operation: self,
            bound_field: &self[id],
            id,
        }
    }

    pub fn walk_definition<'a>(
        &'a self,
        schema: SchemaWalker<'a, ()>,
        definition_id: BoundAnyFieldDefinitionId,
    ) -> BoundAnyFieldDefinitionWalker<'a> {
        match &self[definition_id] {
            BoundAnyFieldDefinition::TypeName(definition) => BoundAnyFieldDefinitionWalker::TypeName(definition),
            BoundAnyFieldDefinition::Field(definition) => {
                BoundAnyFieldDefinitionWalker::Field(BoundFieldDefinitionWalker {
                    field: schema.walk(definition.field_id),
                    definition,
                })
            }
        }
    }
}
