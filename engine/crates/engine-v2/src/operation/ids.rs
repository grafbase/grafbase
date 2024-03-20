use id_newtypes::IdRange;
use schema::InputValueDefinitionId;

use super::{
    BoundField, BoundFieldArgument, BoundFragment, BoundFragmentSpread, BoundInlineFragment, BoundSelectionSet,
    Operation, QueryInputKeyValueId, QueryInputObjectFieldValueId, QueryInputValue, QueryInputValueId,
    VariableDefinition,
};

id_newtypes::U16! {
    Operation.fields[BoundFieldId] => BoundField | unless "Too many fields",
    Operation.selection_sets[BoundSelectionSetId] => BoundSelectionSet | unless "Too many selection sets",
    Operation.fragments[BoundFragmentId] => BoundFragment | unless "Too many fragments",
    Operation.fragment_spreads[BoundFragmentSpreadId] => BoundFragmentSpread | unless "Too many fragment spreads",
    Operation.inline_fragments[BoundInlineFragmentId] => BoundInlineFragment | unless "Too many inline fragments",
    Operation.variable_definitions[VariableDefinitionId] => VariableDefinition | unless "Too many variables",
    Operation.field_arguments[BoundFieldArgumentId] => BoundFieldArgument | unless "Too many arguments",
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
