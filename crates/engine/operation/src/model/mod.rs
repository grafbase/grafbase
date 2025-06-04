mod directive;
mod field;
mod generated;
mod input_value;
mod location;
mod prelude;
mod response_key;
mod selection_set;

use std::sync::Arc;

pub use directive::*;
pub use generated::*;
use grafbase_telemetry::graphql::{GraphqlOperationAttributes, OperationName, OperationType};
pub use input_value::*;
pub use location::*;
pub use response_key::*;
use schema::{InputValueDefinitionId, ObjectDefinition, ObjectDefinitionId, Schema};
pub use selection_set::*;
use walker::{Iter, Walk};

use crate::ComplexityCost;

#[derive(serde::Serialize, serde::Deserialize, id_derives::IndexedFields)]
pub struct Operation {
    pub attributes: OperationAttributes,
    pub root_object_id: ObjectDefinitionId,
    pub root_selection_set_record: SelectionSetRecord,
    pub response_keys: ResponseKeys,
    #[indexed_by(DataFieldId)]
    pub data_fields: Vec<DataFieldRecord>,
    #[indexed_by(TypenameFieldId)]
    pub typename_fields: Vec<TypenameFieldRecord>,
    #[indexed_by(VariableDefinitionId)]
    pub variable_definitions: Vec<VariableDefinitionRecord>,
    #[indexed_by(FieldArgumentId)]
    pub field_arguments: Vec<FieldArgumentRecord>,
    #[indexed_by(InlineFragmentId)]
    pub inline_fragments: Vec<InlineFragmentRecord>,
    #[indexed_by(FragmentSpreadId)]
    pub fragment_spreads: Vec<FragmentSpreadRecord>,
    #[indexed_by(FragmentId)]
    pub fragments: Vec<FragmentRecord>,
    pub query_input_values: QueryInputValues,
    #[indexed_by(SelectionIdSharedVecId)]
    pub shared_selection_ids: Vec<SelectionId>,
}

id_newtypes::forward_with_range! {
    impl Index<QueryInputValueId, Output = QueryInputValueRecord> for Operation.query_input_values,
    impl Index<QueryInputObjectFieldValueId, Output = (InputValueDefinitionId, QueryInputValueRecord)> for Operation.query_input_values,
    impl Index<QueryInputKeyValueId, Output = (String, QueryInputValueRecord)> for Operation.query_input_values,
}

/// The set of Operation attributes that can be cached and kept in metrics/traces
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct OperationAttributes {
    pub ty: OperationType,
    pub name: OperationName,
    pub sanitized_query: Arc<str>,
}

impl OperationAttributes {
    pub fn with_complexity_cost(self, complexity_cost: Option<ComplexityCost>) -> GraphqlOperationAttributes {
        GraphqlOperationAttributes {
            ty: self.ty,
            name: self.name,
            sanitized_query: self.sanitized_query,
            complexity_cost: complexity_cost.map(|c| c.0),
        }
    }
}

#[derive(id_derives::IndexedFields)]
pub struct Variables {
    pub input_values: VariableInputValues,
    #[indexed_by(VariableDefinitionId)]
    pub definition_to_value: Vec<VariableValueRecord>,
}

#[derive(Clone)]
pub enum VariableValueRecord {
    Undefined,
    Provided(VariableInputValueId),
    DefaultValue(QueryInputValueId),
}

impl<T> std::ops::Index<T> for Variables
where
    VariableInputValues: std::ops::Index<T>,
{
    type Output = <VariableInputValues as std::ops::Index<T>>::Output;

    fn index(&self, index: T) -> &Self::Output {
        &self.input_values[index]
    }
}

#[derive(Clone, Copy)]
pub struct OperationContext<'a> {
    pub schema: &'a Schema,
    pub operation: &'a Operation,
}

impl<'a> From<OperationContext<'a>> for &'a Schema {
    fn from(ctx: OperationContext<'a>) -> Self {
        ctx.schema
    }
}

impl<'a> From<OperationContext<'a>> for &'a ResponseKeys {
    fn from(ctx: OperationContext<'a>) -> Self {
        &ctx.operation.response_keys
    }
}

impl<'a> OperationContext<'a> {
    pub fn root_object(&self) -> ObjectDefinition<'a> {
        self.operation.root_object_id.walk(*self)
    }

    pub fn root_selection_set(&self) -> SelectionSet<'a> {
        self.operation.root_selection_set_record.walk(*self)
    }

    pub fn variable_definitions(&self) -> impl Iter<Item = VariableDefinition<'a>> + 'a {
        let ctx = *self;
        (0..self.operation.variable_definitions.len()).map(move |id| VariableDefinition {
            ctx,
            id: VariableDefinitionId::from(id),
        })
    }
}
