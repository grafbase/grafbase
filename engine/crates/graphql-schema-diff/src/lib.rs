#![allow(unused_crate_dependencies)]

mod change;

pub use change::{Change, ChangeKind};

use async_graphql_parser::{types as ast, Positioned};
use async_graphql_value::ConstValue;
use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

type DiffMap<K, V> = HashMap<K, (Option<V>, Option<V>)>;

#[derive(Debug, Default)]
struct AddedRemoved<T> {
    added: T,
    removed: T,
}

fn insert_source<K: Hash + Eq, V>(map: &mut DiffMap<K, V>, key: K, source: V) {
    map.insert(key, (Some(source), None));
}

fn merge_target<K, V>(entry: Entry<'_, K, (Option<V>, Option<V>)>, target: V) {
    entry.or_default().1 = Some(target);
}

#[derive(Default)]
struct DiffState<'a> {
    definitions: Definitions<'a>,
    fields: AddedRemoved<Vec<[&'a str; 2]>>,
    enum_variants: AddedRemoved<Vec<[&'a str; 2]>>,
    union_members: AddedRemoved<Vec<[&'a str; 2]>>,
    arguments: AddedRemoved<Vec<[&'a str; 3]>>,
    argument_default_values: AddedRemoved<Vec<[&'a str; 3]>>,
    argument_default_changed: Vec<[&'a str; 3]>,
    argument_type_changed: Vec<[&'a str; 3]>,
    field_type_changed: Vec<[&'a str; 2]>,
}

macro_rules! definition_kinds {
    ($($camel:ident, $snake:ident);*) => {
            #[derive(Debug, PartialEq, Eq, Clone, Copy)]
            #[repr(u8)]
            enum DefinitionKind {
                $(
                    $camel,
                )*
            }

            #[derive(Default)]
            struct Definitions<'a> {
                $(
                    $snake: AddedRemoved<Vec<&'a str>>,
                )*
            }

            impl<'a> DiffState<'a> {
                fn push_added_type(&mut self, name: &'a str, kind: DefinitionKind) {
                    match kind {
                        $(
                            DefinitionKind::$camel => self.definitions.$snake.added.push(name),
                        )*
                    }
                }

                fn push_removed_type(&mut self, name: &'a str, kind: DefinitionKind) {
                    match kind {
                        $(
                            DefinitionKind::$camel => self.definitions.$snake.removed.push(name),
                        )*
                    }
                }
            }
    }
}

definition_kinds! {
    Directive, directive;
    Enum, r#enum;
    InputObject, input_object;
    Interface, interface;
    Object, object;
    Scalar, scalar;
    Schema, schema;
    Union, union
}

impl DiffState<'_> {
    fn into_changes(self) -> Vec<Change> {
        let DiffState {
            definitions:
                Definitions {
                    directive,
                    r#enum,
                    input_object,
                    interface,
                    object,
                    scalar,
                    schema,
                    union,
                },
            fields,
            enum_variants,
            union_members,
            arguments,
            argument_default_values,
            argument_default_changed,
            field_type_changed,
            argument_type_changed,
        } = self;

        let top_level_changes = [
            (object.added, ChangeKind::AddObjectType),
            (object.removed, ChangeKind::RemoveObjectType),
            (union.added, ChangeKind::AddUnion),
            (union.removed, ChangeKind::RemoveUnion),
            (r#enum.added, ChangeKind::AddEnum),
            (r#enum.removed, ChangeKind::RemoveEnum),
            (scalar.added, ChangeKind::AddScalar),
            (scalar.removed, ChangeKind::RemoveScalar),
            (interface.added, ChangeKind::AddInterface),
            (interface.removed, ChangeKind::RemoveInterface),
            (directive.added, ChangeKind::AddDirectiveDefinition),
            (directive.removed, ChangeKind::RemoveDirectiveDefinition),
            (schema.added, ChangeKind::AddSchemaDefinition),
            (schema.removed, ChangeKind::RemoveSchemaDefinition),
            (input_object.added, ChangeKind::AddInputObject),
            (input_object.removed, ChangeKind::RemoveInputObject),
        ]
        .into_iter()
        .flat_map(|(items, kind)| {
            items.into_iter().map(move |name| Change {
                path: name.to_owned(),
                kind,
            })
        });

        let second_level_changes = [
            (fields.added, ChangeKind::AddField),
            (fields.removed, ChangeKind::RemoveField),
            (field_type_changed, ChangeKind::ChangeFieldType),
            (enum_variants.added, ChangeKind::AddEnumValue),
            (enum_variants.removed, ChangeKind::RemoveEnumValue),
            (union_members.added, ChangeKind::AddUnionMember),
            (union_members.removed, ChangeKind::RemoveUnionMember),
        ]
        .into_iter()
        .flat_map(|(items, kind)| {
            items.into_iter().map(move |path| Change {
                path: path.join("."),
                kind,
            })
        });

        let arguments = [
            (arguments.added, ChangeKind::AddFieldArgument),
            (arguments.removed, ChangeKind::RemoveFieldArgument),
            (argument_default_values.added, ChangeKind::AddFieldArgumentDefault),
            (argument_default_values.removed, ChangeKind::RemoveFieldArgumentDefault),
            (argument_default_changed, ChangeKind::ChangeFieldArgumentDefault),
            (argument_type_changed, ChangeKind::ChangeFieldArgumentType),
        ]
        .into_iter()
        .flat_map(|(items, kind)| {
            items.into_iter().map(move |[parent, field, argument]| Change {
                path: [parent, field, argument].join("."),
                kind,
            })
        });

        let mut changes = top_level_changes
            .chain(second_level_changes)
            .chain(arguments)
            .collect::<Vec<_>>();

        changes.sort();

        changes
    }
}

pub fn diff(source: &str, target: &str) -> Result<Vec<Change>, async_graphql_parser::Error> {
    let source = async_graphql_parser::parse_schema(source)?;
    let target = async_graphql_parser::parse_schema(target)?;

    let mut state = DiffState::default();

    let schema_size_approx = source.definitions.len().max(target.definitions.len());

    let mut types_map: DiffMap<&str, DefinitionKind> = HashMap::with_capacity(schema_size_approx);
    let mut fields_map: DiffMap<[&str; 2], Option<&ast::Type>> = HashMap::with_capacity(schema_size_approx);
    let mut arguments_map: DiffMap<[&str; 3], (&ast::Type, Option<&ConstValue>)> =
        HashMap::with_capacity(schema_size_approx);

    for tpe in &source.definitions {
        match tpe {
            async_graphql_parser::types::TypeSystemDefinition::Schema(_) => {
                insert_source(&mut types_map, ".", DefinitionKind::Schema)
            }
            async_graphql_parser::types::TypeSystemDefinition::Directive(directive_def) => {
                insert_source(&mut types_map, &directive_def.node.name.node, DefinitionKind::Directive);
            }
            async_graphql_parser::types::TypeSystemDefinition::Type(tpe) => {
                let type_name = tpe.node.name.node.as_str();

                match &tpe.node.kind {
                    ast::TypeKind::Scalar => {
                        types_map.insert(type_name, (Some(DefinitionKind::Scalar), None));
                    }
                    ast::TypeKind::Object(obj) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Object), None));

                        for field in &obj.fields {
                            let field_name = field.node.name.node.as_str();

                            insert_source(&mut fields_map, [type_name, field_name], Some(&field.node.ty.node));

                            args_src(&mut arguments_map, type_name, field_name, &field.node.arguments);
                        }
                    }
                    ast::TypeKind::Interface(iface) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Interface), None));

                        for field in &iface.fields {
                            let field_name = field.node.name.node.as_str();

                            insert_source(&mut fields_map, [type_name, field_name], Some(&field.node.ty.node));

                            args_src(&mut arguments_map, type_name, field_name, &field.node.arguments);
                        }
                    }
                    ast::TypeKind::Union(union) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Union), None));

                        for member in &union.members {
                            insert_source(&mut fields_map, [type_name, member.node.as_str()], None);
                        }
                    }
                    ast::TypeKind::Enum(enm) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Enum), None));

                        for value in &enm.values {
                            insert_source(&mut fields_map, [type_name, value.node.value.node.as_str()], None);
                        }
                    }
                    ast::TypeKind::InputObject(input) => {
                        types_map.insert(type_name, (Some(DefinitionKind::InputObject), None));

                        for field in &input.fields {
                            insert_source(
                                &mut fields_map,
                                [type_name, field.node.name.node.as_str()],
                                Some(&field.node.ty.node),
                            );
                        }
                    }
                }
            }
        }
    }

    for tpe in &target.definitions {
        match tpe {
            async_graphql_parser::types::TypeSystemDefinition::Schema(_) => {
                merge_target(types_map.entry("."), DefinitionKind::Schema)
            }
            async_graphql_parser::types::TypeSystemDefinition::Directive(directive_def) => {
                merge_target(
                    types_map.entry(&directive_def.node.name.node),
                    DefinitionKind::Directive,
                );
            }
            async_graphql_parser::types::TypeSystemDefinition::Type(tpe) => {
                let type_name = tpe.node.name.node.as_str();

                match &tpe.node.kind {
                    ast::TypeKind::Scalar => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Scalar);
                    }
                    ast::TypeKind::Object(obj) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Object);

                        for field in &obj.fields {
                            let field_name = field.node.name.node.as_str();

                            merge_target(fields_map.entry([type_name, field_name]), Some(&field.node.ty.node));
                            args_target(&mut arguments_map, type_name, field_name, &field.node.arguments);
                        }
                    }
                    ast::TypeKind::Interface(iface) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Interface);

                        for field in &iface.fields {
                            let field_name = field.node.name.node.as_str();

                            merge_target(fields_map.entry([type_name, field_name]), Some(&field.node.ty.node));
                            args_target(&mut arguments_map, type_name, field_name, &field.node.arguments);
                        }
                    }
                    ast::TypeKind::Union(union) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Union);

                        for member in &union.members {
                            merge_target(fields_map.entry([type_name, member.node.as_str()]), None);
                        }
                    }
                    ast::TypeKind::Enum(enm) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Enum);

                        for value in &enm.values {
                            merge_target(fields_map.entry([type_name, value.node.value.node.as_str()]), None);
                        }
                    }
                    ast::TypeKind::InputObject(input) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::InputObject);

                        for field in &input.fields {
                            merge_target(
                                fields_map.entry([type_name, field.node.name.node.as_str()]),
                                Some(&field.node.ty.node),
                            );
                        }
                    }
                }
            }
        }
    }

    for (name, entries) in &types_map {
        match entries {
            (None, None) => unreachable!(),
            (None, Some(kind)) => state.push_added_type(name, *kind),
            (Some(kind), None) => state.push_removed_type(name, *kind),
            (Some(a), Some(b)) if a != b => {
                state.push_removed_type(name, *a);
                state.push_added_type(name, *b);
            }
            (Some(_), Some(_)) => (),
        }
    }

    for (path @ [type_name, _field_name], (src, target)) in fields_map {
        let kind = match &types_map[type_name] {
            (None, None) => unreachable!(),
            (Some(kind), None) | (None, Some(kind)) => *kind,
            (Some(kind), Some(_)) => *kind,
        };

        match (src, target, kind) {
            (None, None, _) | (_, _, DefinitionKind::Scalar | DefinitionKind::Schema | DefinitionKind::Directive) => {
                unreachable!()
            }
            (None, Some(_), DefinitionKind::Object | DefinitionKind::Interface | DefinitionKind::InputObject) => {
                state.fields.added.push(path)
            }
            (None, Some(_), DefinitionKind::Enum) => state.enum_variants.added.push(path),
            (Some(_), None, DefinitionKind::Enum) => state.enum_variants.removed.push(path),
            (None, Some(_), DefinitionKind::Union) => state.union_members.added.push(path),
            (Some(_), None, DefinitionKind::Union) => state.union_members.removed.push(path),
            (Some(_), None, DefinitionKind::Object | DefinitionKind::Interface | DefinitionKind::InputObject) => {
                state.fields.removed.push(path)
            }
            (
                Some(ty_a),
                Some(ty_b),
                DefinitionKind::Object | DefinitionKind::InputObject | DefinitionKind::Interface,
            ) if ty_a != ty_b => state.field_type_changed.push(path),
            (Some(_), Some(_), _) => (),
        }
    }

    for (path, (src, target)) in arguments_map {
        match (src, target) {
            (None, None) => unreachable!(),
            (None, Some(_)) => state.arguments.added.push(path),
            (Some(_), None) => state.arguments.removed.push(path),
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

    Ok(state.into_changes())
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
