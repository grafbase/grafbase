#![allow(unused_crate_dependencies)]

mod change;

pub use change::{Change, ChangeKind};

use async_graphql_parser::{types as ast, Positioned};
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

struct DiffState<'a> {
    definitions: Definitions<'a>,
    fields: AddedRemoved<Vec<(&'a str, &'a str)>>,
    enum_variants: AddedRemoved<Vec<(&'a str, &'a str)>>,
    union_members: AddedRemoved<Vec<(&'a str, &'a str)>>,
    arguments: AddedRemoved<Vec<[&'a str; 3]>>,
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
            (enum_variants.added, ChangeKind::AddEnumValue),
            (enum_variants.removed, ChangeKind::RemoveEnumValue),
            (union_members.added, ChangeKind::AddUnionMember),
            (union_members.removed, ChangeKind::RemoveUnionMember),
        ]
        .into_iter()
        .flat_map(|(items, kind)| {
            items.into_iter().map(move |(parent, field)| Change {
                path: [parent, field].join("."),
                kind,
            })
        });

        let arguments = arguments
            .added
            .into_iter()
            .map(|path| Change {
                path: path.join("."),
                kind: ChangeKind::AddFieldArgument,
            })
            .chain(arguments.removed.into_iter().map(|path| Change {
                path: path.join("."),
                kind: ChangeKind::RemoveFieldArgument,
            }));

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
    let mut state = DiffState {
        definitions: Default::default(),
        fields: Default::default(),
        enum_variants: Default::default(),
        union_members: Default::default(),
        arguments: Default::default(),
    };

    let schema_size_approx = source.definitions.len().max(target.definitions.len());

    let mut types_map: DiffMap<&str, DefinitionKind> = HashMap::with_capacity(schema_size_approx);
    let mut fields_map: DiffMap<(&str, &str), Option<&ast::Type>> = HashMap::with_capacity(schema_size_approx);
    let mut arguments_map: DiffMap<[&str; 3], &ast::Type> = HashMap::with_capacity(schema_size_approx);

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

                            insert_source(&mut fields_map, (type_name, field_name), Some(&field.node.ty.node));

                            args_src(&mut arguments_map, type_name, field_name, &field.node.arguments);
                        }
                    }
                    ast::TypeKind::Interface(iface) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Interface), None));

                        for field in &iface.fields {
                            let field_name = field.node.name.node.as_str();

                            insert_source(&mut fields_map, (type_name, field_name), Some(&field.node.ty.node));

                            args_src(&mut arguments_map, type_name, field_name, &field.node.arguments);
                        }
                    }
                    ast::TypeKind::Union(union) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Union), None));

                        for member in &union.members {
                            insert_source(&mut fields_map, (type_name, member.node.as_str()), None);
                        }
                    }
                    ast::TypeKind::Enum(enm) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Enum), None));

                        for value in &enm.values {
                            insert_source(&mut fields_map, (type_name, value.node.value.node.as_str()), None);
                        }
                    }
                    ast::TypeKind::InputObject(input) => {
                        types_map.insert(type_name, (Some(DefinitionKind::InputObject), None));

                        for field in &input.fields {
                            insert_source(
                                &mut fields_map,
                                (type_name, field.node.name.node.as_str()),
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
            async_graphql_parser::types::TypeSystemDefinition::Schema(_) => todo!(),
            async_graphql_parser::types::TypeSystemDefinition::Directive(_) => todo!(),
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

                            merge_target(fields_map.entry((type_name, field_name)), Some(&field.node.ty.node));
                            args_target(&mut arguments_map, type_name, field_name, &field.node.arguments);
                        }
                    }
                    ast::TypeKind::Interface(iface) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Interface);

                        for field in &iface.fields {
                            let field_name = field.node.name.node.as_str();

                            merge_target(fields_map.entry((type_name, field_name)), Some(&field.node.ty.node));
                            args_target(&mut arguments_map, type_name, field_name, &field.node.arguments);
                        }
                    }
                    ast::TypeKind::Union(union) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Union);

                        for member in &union.members {
                            merge_target(fields_map.entry((type_name, member.node.as_str())), None);
                        }
                    }
                    ast::TypeKind::Enum(enm) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Enum);

                        for value in &enm.values {
                            merge_target(fields_map.entry((type_name, value.node.value.node.as_str())), None);
                        }
                    }
                    ast::TypeKind::InputObject(input) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::InputObject);

                        for field in &input.fields {
                            merge_target(
                                fields_map.entry((type_name, field.node.name.node.as_str())),
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

    for (path @ (type_name, _field_name), (src, target)) in fields_map {
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
            (Some(_), Some(_), _) => (),
        }
    }

    for (path, (src, target)) in arguments_map {
        match (src, target) {
            (None, None) => unreachable!(),
            (None, Some(_)) => state.arguments.added.push(path),
            (Some(_), None) => state.arguments.removed.push(path),
            (Some(_), Some(_)) => (),
        }
    }

    Ok(state.into_changes())
}

fn args_src<'a>(
    arguments_map: &mut DiffMap<[&'a str; 3], &'a ast::Type>,
    parent: &'a str,
    field: &'a str,
    args: &'a [Positioned<ast::InputValueDefinition>],
) {
    for arg in args {
        insert_source(arguments_map, [parent, field, &arg.node.name.node], &arg.node.ty.node)
    }
}

fn args_target<'a>(
    arguments_map: &mut DiffMap<[&'a str; 3], &'a ast::Type>,
    parent: &'a str,
    field: &'a str,
    args: &'a [Positioned<ast::InputValueDefinition>],
) {
    for arg in args {
        merge_target(
            arguments_map.entry([parent, field, &arg.node.name.node]),
            &arg.node.ty.node,
        )
    }
}
