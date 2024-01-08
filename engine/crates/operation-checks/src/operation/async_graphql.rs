use super::*;
use async_graphql_parser::types::ExecutableDocument;

impl From<ExecutableDocument> for super::Operation {
    fn from(value: ExecutableDocument) -> Self {
        let mut selection_id_counter = 0usize;
        let mut fragments = HashMap::with_capacity(value.fragments.len());
        let mut selections = Vec::new();

        for (name, fragment) in &value.fragments {
            let selection_id = SelectionId(selection_id_counter);
            selection_id_counter += 1;

            for item in fragment.node.selection_set.node.items.iter() {
                let item = ingest_selection(&mut selection_id_counter, &item.node, &mut selections);
                selections.push((selection_id, item));
            }
            let type_condition = fragment.node.type_condition.node.on.node.to_string();
            fragments.insert(
                name.to_string(),
                Fragment {
                    type_condition,
                    selection: selection_id,
                },
            );
        }

        let operation = &value.operations.iter().next().unwrap().1;

        let operation_type = match operation.node.ty {
            async_graphql_parser::types::OperationType::Query => super::OperationType::Query,
            async_graphql_parser::types::OperationType::Mutation => super::OperationType::Mutation,
            async_graphql_parser::types::OperationType::Subscription => super::OperationType::Subscription,
        };

        let root_selection = SelectionId(selection_id_counter);
        selection_id_counter += 1;

        for item in &operation.node.selection_set.node.items {
            let item = ingest_selection(&mut selection_id_counter, &item.node, &mut selections);
            selections.push((root_selection, item));
        }

        selections.sort_by_key(|(parent_id, _)| *parent_id);

        super::Operation {
            fragments,
            operation_type,
            root_selection,
            selections,
        }
    }
}

fn ingest_selection(
    counter: &mut usize,
    selection: &async_graphql_parser::types::Selection,
    selections: &mut Vec<(SelectionId, super::Selection)>,
) -> super::Selection {
    match selection {
        async_graphql_parser::types::Selection::Field(field) => {
            let subselection = if field.node.selection_set.node.items.is_empty() {
                None
            } else {
                let subselection_id = SelectionId(*counter);
                *counter += 1;

                for item in &field.node.selection_set.node.items {
                    let item = ingest_selection(counter, &item.node, selections);
                    selections.push((subselection_id, item));
                }

                Some(subselection_id)
            };

            super::Selection::Field {
                field_name: field.node.name.node.to_string(),
                arguments: field
                    .node
                    .arguments
                    .iter()
                    .map(|(name, _)| name.node.to_string())
                    .collect(),
                subselection,
            }
        }
        async_graphql_parser::types::Selection::FragmentSpread(fragment_name) => super::Selection::FragmentSpread {
            fragment_name: fragment_name.node.fragment_name.node.to_string(),
        },
        async_graphql_parser::types::Selection::InlineFragment(inline_fragment) => {
            let selection_id = SelectionId(*counter);
            *counter += 1;

            for item in &inline_fragment.node.selection_set.node.items {
                let item = ingest_selection(counter, &item.node, selections);
                selections.push((selection_id, item));
            }

            super::Selection::InlineFragment {
                on: inline_fragment
                    .node
                    .type_condition
                    .as_ref()
                    .map(|on| on.node.on.node.to_string()),
                selection: selection_id,
            }
        }
    }
}
