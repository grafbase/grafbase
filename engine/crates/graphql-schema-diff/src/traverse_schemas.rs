use crate::{ast, ConstValue, DefinitionKind, DiffMap, DiffState, Positioned};
use std::{collections::hash_map::Entry, hash::Hash};

/// Traverse the source and target schemas, populating the `DiffState`.
pub(crate) fn traverse_schemas<'a>([source, target]: [Option<&'a ast::ServiceDocument>; 2], state: &mut DiffState<'a>) {
    let [source_definitions_len, target_definitions_len] =
        [source, target].map(|schema| schema.map(|schema| schema.definitions.len()).unwrap_or_default());
    let schema_size_approx = source_definitions_len.max(target_definitions_len);
    state.types_map.reserve(schema_size_approx);
    state.fields_map.reserve(schema_size_approx);

    if let Some(source) = source {
        traverse_source(source, state);
    }

    if let Some(target) = target {
        traverse_target(target, state);
    }
}

fn traverse_source<'a>(source: &'a ast::ServiceDocument, state: &mut DiffState<'a>) {
    for tpe in &source.definitions {
        match tpe {
            async_graphql_parser::types::TypeSystemDefinition::Schema(def) => {
                state.schema_definition_map[0] = Some(&def.node);
            }
            async_graphql_parser::types::TypeSystemDefinition::Directive(directive_def) => {
                insert_source(
                    &mut state.types_map,
                    &directive_def.node.name.node,
                    DefinitionKind::Directive,
                );
            }
            async_graphql_parser::types::TypeSystemDefinition::Type(tpe) => {
                let type_name = tpe.node.name.node.as_str();

                match &tpe.node.kind {
                    ast::TypeKind::Scalar => {
                        state.types_map.insert(type_name, (Some(DefinitionKind::Scalar), None));
                    }
                    ast::TypeKind::Object(obj) => {
                        state.types_map.insert(type_name, (Some(DefinitionKind::Object), None));
                        insert_source(&mut state.interface_impls, type_name, &obj.implements);

                        for field in &obj.fields {
                            let field_name = field.node.name.node.as_str();

                            insert_source(
                                &mut state.fields_map,
                                [type_name, field_name],
                                Some(&field.node.ty.node),
                            );

                            fill_args_src(&mut state.arguments_map, type_name, field_name, &field.node.arguments);
                        }
                    }
                    ast::TypeKind::Interface(iface) => {
                        state
                            .types_map
                            .insert(type_name, (Some(DefinitionKind::Interface), None));
                        insert_source(&mut state.interface_impls, type_name, &iface.implements);

                        for field in &iface.fields {
                            let field_name = field.node.name.node.as_str();

                            insert_source(
                                &mut state.fields_map,
                                [type_name, field_name],
                                Some(&field.node.ty.node),
                            );

                            fill_args_src(&mut state.arguments_map, type_name, field_name, &field.node.arguments);
                        }
                    }
                    ast::TypeKind::Union(union) => {
                        state.types_map.insert(type_name, (Some(DefinitionKind::Union), None));

                        for member in &union.members {
                            insert_source(&mut state.fields_map, [type_name, member.node.as_str()], None);
                        }
                    }
                    ast::TypeKind::Enum(enm) => {
                        state.types_map.insert(type_name, (Some(DefinitionKind::Enum), None));

                        for value in &enm.values {
                            insert_source(&mut state.fields_map, [type_name, value.node.value.node.as_str()], None);
                        }
                    }
                    ast::TypeKind::InputObject(input) => {
                        state
                            .types_map
                            .insert(type_name, (Some(DefinitionKind::InputObject), None));

                        for field in &input.fields {
                            let field_name = field.node.name.node.as_str();
                            insert_source(
                                &mut state.fields_map,
                                [type_name, field_name],
                                Some(&field.node.ty.node),
                            );
                        }
                    }
                }
            }
        }
    }
}

fn traverse_target<'a>(target: &'a ast::ServiceDocument, state: &mut DiffState<'a>) {
    for tpe in &target.definitions {
        match tpe {
            async_graphql_parser::types::TypeSystemDefinition::Schema(def) => {
                state.schema_definition_map[1] = Some(&def.node);
            }
            async_graphql_parser::types::TypeSystemDefinition::Directive(directive_def) => {
                merge_target(
                    state.types_map.entry(&directive_def.node.name.node),
                    DefinitionKind::Directive,
                );
            }
            async_graphql_parser::types::TypeSystemDefinition::Type(tpe) => {
                let type_name = tpe.node.name.node.as_str();

                match &tpe.node.kind {
                    ast::TypeKind::Scalar => {
                        state.types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Scalar);
                    }
                    ast::TypeKind::Object(obj) => {
                        state.types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Object);
                        merge_target(state.interface_impls.entry(type_name), &obj.implements);

                        for field in &obj.fields {
                            let field_name = field.node.name.node.as_str();

                            merge_target(
                                state.fields_map.entry([type_name, field_name]),
                                Some(&field.node.ty.node),
                            );
                            args_target(&mut state.arguments_map, type_name, field_name, &field.node.arguments);
                        }
                    }
                    ast::TypeKind::Interface(iface) => {
                        state.types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Interface);
                        merge_target(state.interface_impls.entry(type_name), &iface.implements);

                        for field in &iface.fields {
                            let field_name = field.node.name.node.as_str();

                            merge_target(
                                state.fields_map.entry([type_name, field_name]),
                                Some(&field.node.ty.node),
                            );
                            args_target(&mut state.arguments_map, type_name, field_name, &field.node.arguments);
                        }
                    }
                    ast::TypeKind::Union(union) => {
                        state.types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Union);

                        for member in &union.members {
                            merge_target(state.fields_map.entry([type_name, member.node.as_str()]), None);
                        }
                    }
                    ast::TypeKind::Enum(enm) => {
                        state.types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Enum);

                        for value in &enm.values {
                            merge_target(
                                state.fields_map.entry([type_name, value.node.value.node.as_str()]),
                                None,
                            );
                        }
                    }
                    ast::TypeKind::InputObject(input) => {
                        state.types_map.entry(type_name).or_default().1 = Some(DefinitionKind::InputObject);

                        for field in &input.fields {
                            merge_target(
                                state.fields_map.entry([type_name, field.node.name.node.as_str()]),
                                Some(&field.node.ty.node),
                            );
                        }
                    }
                }
            }
        }
    }
}

// Insert the arguments of a field into the DiffState.
fn fill_args_src<'a>(
    arguments_map: &mut DiffMap<[&'a str; 3], (&'a ast::Type, Option<&'a ConstValue>)>,
    parent: &'a str,
    field: &'a str,
    args: &'a [Positioned<ast::InputValueDefinition>],
) {
    for arg in args {
        insert_source(
            arguments_map,
            [parent, field, &arg.node.name.node],
            (&arg.node.ty.node, arg.node.default_value.as_ref().map(|pos| &pos.node)),
        )
    }
}

// Merge the arguments of a field in the target schema into the DiffState.
fn args_target<'a>(
    arguments_map: &mut DiffMap<[&'a str; 3], (&'a ast::Type, Option<&'a ConstValue>)>,
    parent: &'a str,
    field: &'a str,
    args: &'a [Positioned<ast::InputValueDefinition>],
) {
    for arg in args {
        merge_target(
            arguments_map.entry([parent, field, &arg.node.name.node]),
            (&arg.node.ty.node, arg.node.default_value.as_ref().map(|pos| &pos.node)),
        )
    }
}

fn insert_source<K: Hash + Eq, V>(map: &mut DiffMap<K, V>, key: K, source: V) {
    map.insert(key, (Some(source), None));
}

fn merge_target<K, V>(entry: Entry<'_, K, (Option<V>, Option<V>)>, target: V) {
    entry.or_default().1 = Some(target);
}
