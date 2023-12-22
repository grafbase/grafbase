use crate::*;

pub(crate) fn traverse_schemas<'a>([source, target]: [&'a ast::ServiceDocument; 2], state: &mut DiffState<'a>) {
    let schema_size_approx = source.definitions.len().max(target.definitions.len());
    state.types_map.reserve(schema_size_approx);
    state.fields_map.reserve(schema_size_approx);

    traverse_source(source, state);
    traverse_target(target, state);

    for (path @ [type_name, _field_name], (src, target)) in &state.fields_map {
        let parent = &state.types_map[type_name];
        let parent_is_gone = || matches!(parent, (Some(_), None));

        if matches!(parent, (Some(a), Some(b)) if a != b) {
            continue; // so we don't falsely interpret same name as field type change
        }

        let kind = match parent {
            (None, None) => unreachable!(),
            (Some(kind), None) | (None, Some(kind)) => *kind,
            (Some(kind), Some(_)) => *kind,
        };

        match (src, target, kind) {
            (None, None, _) | (_, _, DefinitionKind::Scalar | DefinitionKind::Directive) => {
                unreachable!()
            }
            (None, Some(_), DefinitionKind::Object | DefinitionKind::Interface | DefinitionKind::InputObject) => {
                state.fields.added.push(*path)
            }
            (None, Some(_), DefinitionKind::Enum) => state.enum_variants.added.push(*path),
            (Some(_), None, DefinitionKind::Enum) if !parent_is_gone() => state.enum_variants.removed.push(*path),
            (None, Some(_), DefinitionKind::Union) => state.union_members.added.push(*path),
            (Some(_), None, DefinitionKind::Union) if !parent_is_gone() => state.union_members.removed.push(*path),
            (Some(_), None, DefinitionKind::Object | DefinitionKind::Interface | DefinitionKind::InputObject)
                if !parent_is_gone() =>
            {
                state.fields.removed.push(*path)
            }
            (
                Some(ty_a),
                Some(ty_b),
                DefinitionKind::Object | DefinitionKind::InputObject | DefinitionKind::Interface,
            ) if ty_a != ty_b => state.field_type_changed.push(*path),
            (Some(_), None, _) => (),
            (Some(_), Some(_), _) => (),
        }
    }

    for (path @ [type_name, field_name, _arg_name], (src, target)) in &state.arguments_map {
        let path = *path;
        let parent_is_gone = || matches!(&state.fields_map[&[*type_name, *field_name]], (Some(_), None));

        match (src, target) {
            (None, None) => unreachable!(),
            (None, Some(_)) => state.arguments.added.push(path),
            (Some(_), None) if !parent_is_gone() => state.arguments.removed.push(path),
            (Some(_), None) => (),
            (Some((src_type, src_default)), Some((target_type, target_default))) => {
                if src_type != target_type {
                    state.argument_type_changed.push(path);
                }

                match (src_default, target_default) {
                    (None, Some(_)) => state.argument_default_values.added.push(path),
                    (Some(_), None) => state.argument_default_values.removed.push(path),
                    (Some(a), Some(b)) if a != b => state.argument_default_changed.push(path),
                    _ => (),
                }
            }
        }
    }
}

fn traverse_source<'a>(source: &'a ast::ServiceDocument, state: &mut DiffState<'a>) {
    for tpe in &source.definitions {
        match tpe {
            async_graphql_parser::types::TypeSystemDefinition::Schema(def) => {
                state.schema_definition_map.0 = Some(&def.node);
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

                        for field in &obj.fields {
                            let field_name = field.node.name.node.as_str();

                            insert_source(
                                &mut state.fields_map,
                                [type_name, field_name],
                                Some(&field.node.ty.node),
                            );

                            args_src(&mut state.arguments_map, type_name, field_name, &field.node.arguments);
                        }
                    }
                    ast::TypeKind::Interface(iface) => {
                        state
                            .types_map
                            .insert(type_name, (Some(DefinitionKind::Interface), None));

                        for field in &iface.fields {
                            let field_name = field.node.name.node.as_str();

                            insert_source(
                                &mut state.fields_map,
                                [type_name, field_name],
                                Some(&field.node.ty.node),
                            );

                            args_src(&mut state.arguments_map, type_name, field_name, &field.node.arguments);
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
                state.schema_definition_map.1 = Some(&def.node);
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

fn args_src<'a>(
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
