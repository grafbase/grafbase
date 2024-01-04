use async_graphql_parser::types::{ExecutableDocument, ServiceDocument};
use std::collections::HashMap;

use super::{Fragment, SchemaField, SelectionId};

impl From<ExecutableDocument> for super::Query {
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

        super::Query {
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

impl From<ServiceDocument> for super::Schema {
    fn from(value: ServiceDocument) -> Self {
        let mut fields: Vec<SchemaField> = Vec::new();
        let mut query_type_name: Option<String> = None;
        let mut mutation_type_name = None;
        let mut subscription_type_name = None;

        for definition in value.definitions {
            match definition {
                async_graphql_parser::types::TypeSystemDefinition::Schema(schema_def) => {
                    if let Some(query) = schema_def.node.query {
                        query_type_name = Some(query.node.to_string());
                    }

                    if let Some(mutation) = schema_def.node.mutation {
                        mutation_type_name = Some(mutation.node.to_string());
                    }

                    if let Some(subscription) = schema_def.node.subscription {
                        subscription_type_name = Some(subscription.node.to_string());
                    }
                }
                async_graphql_parser::types::TypeSystemDefinition::Directive(_) => (),
                async_graphql_parser::types::TypeSystemDefinition::Type(typedef) => {
                    let type_name = typedef.node.name.node;

                    match typedef.node.kind {
                        async_graphql_parser::types::TypeKind::Enum(_)
                        | async_graphql_parser::types::TypeKind::Scalar
                        | async_graphql_parser::types::TypeKind::Union(_) => (),
                        async_graphql_parser::types::TypeKind::Object(obj) => {
                            for field in &obj.fields {
                                fields.push(SchemaField {
                                    type_name: type_name.to_string(),
                                    field_name: field.node.name.node.to_string(),
                                    base_type: extract_type_name(&field.node.ty.node.base),
                                });
                            }
                        }
                        async_graphql_parser::types::TypeKind::Interface(iface) => {
                            for field in &iface.fields {
                                fields.push(SchemaField {
                                    type_name: type_name.to_string(),
                                    field_name: field.node.name.node.to_string(),
                                    base_type: extract_type_name(&field.node.ty.node.base),
                                });
                            }
                        }
                        async_graphql_parser::types::TypeKind::InputObject(input_obj) => {
                            for field in &input_obj.fields {
                                fields.push(SchemaField {
                                    type_name: type_name.to_string(),
                                    field_name: field.node.name.node.to_string(),
                                    base_type: extract_type_name(&field.node.ty.node.base),
                                });
                            }
                        }
                    }
                }
            }
        }

        fields.sort();

        super::Schema {
            fields,
            query_type_name: query_type_name.unwrap_or_else(|| "Query".to_owned()),
            mutation_type_name: mutation_type_name.unwrap_or_else(|| "Mutation".to_owned()),
            subscription_type_name: subscription_type_name.unwrap_or_else(|| "Subscription".to_owned()),
        }
    }
}

fn extract_type_name(ty: &async_graphql_parser::types::BaseType) -> String {
    match ty {
        async_graphql_parser::types::BaseType::Named(name) => name.to_string(),
        async_graphql_parser::types::BaseType::List(inner) => extract_type_name(&inner.base),
    }
}
