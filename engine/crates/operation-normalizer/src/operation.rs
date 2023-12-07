use std::collections::HashMap;

use graphql_parser::query::OperationDefinition;

use crate::{directives, selection_set};

pub(super) fn normalize<'a>(
    operation: &mut OperationDefinition<'a, &'a str>,
    used_fragments: &mut HashMap<String, bool>,
) -> anyhow::Result<()> {
    match operation {
        OperationDefinition::SelectionSet(selection_set) => {
            selection_set::normalize(selection_set, used_fragments, true);
        }
        OperationDefinition::Query(query) => {
            directives::normalize(&mut query.directives);
            selection_set::normalize(&mut query.selection_set, used_fragments, true);

            query.variable_definitions.sort_by(|a, b| a.name.cmp(b.name));
        }
        OperationDefinition::Mutation(mutation) => {
            directives::normalize(&mut mutation.directives);
            selection_set::normalize(&mut mutation.selection_set, used_fragments, true);

            mutation.variable_definitions.sort_by(|a, b| a.name.cmp(b.name));
        }
        OperationDefinition::Subscription(subscription) => {
            directives::normalize(&mut subscription.directives);
            selection_set::normalize(&mut subscription.selection_set, used_fragments, true);

            subscription.variable_definitions.sort_by(|a, b| a.name.cmp(b.name));
        }
    }

    Ok(())
}
