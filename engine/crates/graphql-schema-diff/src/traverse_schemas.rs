use crate::{ast, DefinitionKind, DiffMap, DiffState};
use std::{collections::hash_map::Entry, hash::Hash};

/// Traverse the source and target schemas, populating the `DiffState`.
pub(crate) fn traverse_schemas<'a>(
    [source, target]: [Option<&'a ast::TypeSystemDocument>; 2],
    state: &mut DiffState<'a>,
) {
    let [source_definitions_len, target_definitions_len] =
        [source, target].map(|schema| schema.map(|schema| schema.definitions().len()).unwrap_or_default());
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

fn traverse_source<'a>(source: &'a ast::TypeSystemDocument, state: &mut DiffState<'a>) {
    for tpe in source.definitions() {
        match tpe {
            ast::Definition::Schema(def) | ast::Definition::SchemaExtension(def) => {
                state.schema_definition_map[0] = Some(def);
            }
            ast::Definition::Directive(directive_def) => {
                insert_source(&mut state.types_map, directive_def.name(), DefinitionKind::Directive);
            }
            ast::Definition::Type(tpe) | ast::Definition::TypeExtension(tpe) => {
                let type_name = tpe.name();

                match &tpe {
                    ast::TypeDefinition::Scalar(_) => {
                        state.types_map.insert(type_name, (Some(DefinitionKind::Scalar), None));
                    }
                    ast::TypeDefinition::Object(obj) => {
                        state.types_map.insert(type_name, (Some(DefinitionKind::Object), None));
                        insert_source(
                            &mut state.interface_impls,
                            type_name,
                            obj.implements_interfaces().collect(),
                        );

                        for field in obj.fields() {
                            let field_name = field.name();

                            insert_source(&mut state.fields_map, [type_name, field_name], Some(field.ty()));

                            let mut args = field.arguments();
                            fill_args_src(&mut state.arguments_map, type_name, field_name, &mut args);
                        }
                    }
                    ast::TypeDefinition::Interface(iface) => {
                        state
                            .types_map
                            .insert(type_name, (Some(DefinitionKind::Interface), None));
                        insert_source(
                            &mut state.interface_impls,
                            type_name,
                            iface.implements_interfaces().collect(),
                        );

                        for field in iface.fields() {
                            let field_name = field.name();

                            insert_source(&mut state.fields_map, [type_name, field_name], Some(field.ty()));

                            fill_args_src(&mut state.arguments_map, type_name, field_name, &mut field.arguments());
                        }
                    }
                    ast::TypeDefinition::Union(union) => {
                        state.types_map.insert(type_name, (Some(DefinitionKind::Union), None));

                        for member in union.members() {
                            insert_source(&mut state.fields_map, [type_name, member], None);
                        }
                    }
                    ast::TypeDefinition::Enum(enm) => {
                        state.types_map.insert(type_name, (Some(DefinitionKind::Enum), None));

                        for value in enm.values() {
                            insert_source(&mut state.fields_map, [type_name, value.value()], None);
                        }
                    }
                    ast::TypeDefinition::InputObject(input) => {
                        state
                            .types_map
                            .insert(type_name, (Some(DefinitionKind::InputObject), None));

                        for field in input.fields() {
                            insert_source(&mut state.fields_map, [type_name, field.name()], Some(field.ty()));
                        }
                    }
                }
            }
        }
    }
}

fn traverse_target<'a>(target: &'a ast::TypeSystemDocument, state: &mut DiffState<'a>) {
    for tpe in target.definitions() {
        match tpe {
            ast::Definition::Schema(def) | ast::Definition::SchemaExtension(def) => {
                state.schema_definition_map[1] = Some(def);
            }
            ast::Definition::Directive(directive_def) => {
                merge_target(state.types_map.entry(directive_def.name()), DefinitionKind::Directive);
            }
            ast::Definition::Type(tpe) | ast::Definition::TypeExtension(tpe) => {
                let type_name = tpe.name();

                match tpe {
                    ast::TypeDefinition::Scalar(_) => {
                        state.types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Scalar);
                    }
                    ast::TypeDefinition::Object(obj) => {
                        state.types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Object);
                        merge_target(
                            state.interface_impls.entry(type_name),
                            obj.implements_interfaces().collect(),
                        );

                        for field in obj.fields() {
                            merge_target(state.fields_map.entry([type_name, field.name()]), Some(field.ty()));
                            let mut args = field.arguments();
                            args_target(&mut state.arguments_map, type_name, field.name(), &mut args);
                        }
                    }
                    ast::TypeDefinition::Interface(iface) => {
                        state.types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Interface);
                        merge_target(
                            state.interface_impls.entry(type_name),
                            iface.implements_interfaces().collect(),
                        );

                        for field in iface.fields() {
                            let field_name = field.name();

                            merge_target(state.fields_map.entry([type_name, field_name]), Some(field.ty()));
                            args_target(&mut state.arguments_map, type_name, field_name, &mut field.arguments());
                        }
                    }
                    ast::TypeDefinition::Union(union) => {
                        state.types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Union);

                        for member in union.members() {
                            merge_target(state.fields_map.entry([type_name, member]), None);
                        }
                    }
                    ast::TypeDefinition::Enum(enm) => {
                        state.types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Enum);

                        for value in enm.values() {
                            merge_target(state.fields_map.entry([type_name, value.value()]), None);
                        }
                    }
                    ast::TypeDefinition::InputObject(input) => {
                        state.types_map.entry(type_name).or_default().1 = Some(DefinitionKind::InputObject);

                        for field in input.fields() {
                            merge_target(state.fields_map.entry([type_name, field.name()]), Some(field.ty()));
                        }
                    }
                }
            }
        }
    }
}

// Insert the arguments of a field into the DiffState.
fn fill_args_src<'a>(
    arguments_map: &mut DiffMap<[&'a str; 3], (ast::Type<'a>, Option<ast::Value<'a>>)>,
    parent: &'a str,
    field: &'a str,
    args: &mut (dyn Iterator<Item = ast::InputValueDefinition<'a>> + 'a),
) {
    for arg in args {
        insert_source(
            arguments_map,
            [parent, field, (arg.name())],
            (arg.ty(), arg.default_value()),
        )
    }
}

// Merge the arguments of a field in the target schema into the DiffState.
fn args_target<'a>(
    arguments_map: &mut DiffMap<[&'a str; 3], (ast::Type<'a>, Option<ast::Value<'a>>)>,
    parent: &'a str,
    field: &'a str,
    args: &mut (dyn Iterator<Item = ast::InputValueDefinition<'a>> + 'a),
) {
    for arg in args {
        merge_target(
            arguments_map.entry([parent, field, arg.name()]),
            (arg.ty(), arg.default_value()),
        )
    }
}

fn insert_source<K: Hash + Eq, V>(map: &mut DiffMap<K, V>, key: K, source: V) {
    map.insert(key, (Some(source), None));
}

fn merge_target<K, V>(entry: Entry<'_, K, (Option<V>, Option<V>)>, target: V) {
    entry.or_default().1 = Some(target);
}
