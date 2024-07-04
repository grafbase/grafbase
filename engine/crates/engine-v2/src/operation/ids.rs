use id_newtypes::IdRange;
use schema::InputValueDefinitionId;
use std::num::NonZeroU16;

use super::{
    Condition, Field, FieldArgument, Fragment, FragmentSpread, InlineFragment, Operation, Plan, QueryInputKeyValueId,
    QueryInputObjectFieldValueId, QueryInputValue, QueryInputValueId, SelectionSet, VariableDefinition,
};

id_newtypes::NonZeroU16! {
    Operation.fields[FieldId] => Field,
    Operation.selection_sets[SelectionSetId] => SelectionSet,
    Operation.fragments[FragmentId] => Fragment,
    Operation.fragment_spreads[FragmentSpreadId] => FragmentSpread,
    Operation.inline_fragments[InlineFragmentId] => InlineFragment,
    Operation.variable_definitions[VariableDefinitionId] => VariableDefinition,
    Operation.field_arguments[FieldArgumentId] => FieldArgument,
    Operation.plans[PlanId] => Plan,
    Operation.conditions[ConditionId] => Condition,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct EntityLocation(NonZeroU16);

impl From<usize> for EntityLocation {
    fn from(value: usize) -> Self {
        Self(
            u16::try_from(value)
                .ok()
                .and_then(|value| NonZeroU16::new(value + 1))
                .expect("Too many entity locations"),
        )
    }
}

impl From<EntityLocation> for usize {
    fn from(value: EntityLocation) -> Self {
        usize::from(value.0.get()) - 1
    }
}

impl std::ops::Index<QueryInputValueId> for Operation {
    type Output = QueryInputValue;

    fn index(&self, index: QueryInputValueId) -> &Self::Output {
        &self.query_input_values[index]
    }
}

impl std::ops::Index<IdRange<QueryInputValueId>> for Operation {
    type Output = [QueryInputValue];

    fn index(&self, index: IdRange<QueryInputValueId>) -> &Self::Output {
        &self.query_input_values[index]
    }
}

impl std::ops::Index<QueryInputKeyValueId> for Operation {
    type Output = (String, QueryInputValue);

    fn index(&self, index: QueryInputKeyValueId) -> &Self::Output {
        &self.query_input_values[index]
    }
}

impl std::ops::Index<IdRange<QueryInputKeyValueId>> for Operation {
    type Output = [(String, QueryInputValue)];

    fn index(&self, index: IdRange<QueryInputKeyValueId>) -> &Self::Output {
        &self.query_input_values[index]
    }
}

impl std::ops::Index<QueryInputObjectFieldValueId> for Operation {
    type Output = (InputValueDefinitionId, QueryInputValue);

    fn index(&self, index: QueryInputObjectFieldValueId) -> &Self::Output {
        &self.query_input_values[index]
    }
}

impl std::ops::Index<IdRange<QueryInputObjectFieldValueId>> for Operation {
    type Output = [(InputValueDefinitionId, QueryInputValue)];

    fn index(&self, index: IdRange<QueryInputObjectFieldValueId>) -> &Self::Output {
        &self.query_input_values[index]
    }
}
