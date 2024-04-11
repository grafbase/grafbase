use id_newtypes::IdRange;
use schema::InputValueDefinitionId;

use super::{
    Field, FieldArgument, Fragment, FragmentSpread, InlineFragment, Operation, QueryInputKeyValueId,
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
