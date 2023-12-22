#![allow(unused_crate_dependencies)]

mod change;

pub use change::{Change, ChangeKind};

use async_graphql_parser::types as ast;
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
    source: &'a ast::ServiceDocument,
    target: &'a ast::ServiceDocument,
    definitions: Definitions<'a>,
    fields: AddedRemoved<Vec<(&'a str, &'a str)>>,
    enum_variants: AddedRemoved<Vec<(&'a str, &'a str)>>,
    union_members: AddedRemoved<Vec<(&'a str, &'a str)>>,
    arguments: AddedRemoved<Vec<(&'a str, &'a str, &'a str)>>,
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
        let Definitions {
            directive,
            r#enum,
            input_object,
            interface,
            object,
            scalar,
            schema,
            union,
        } = self.definitions;

        let mut changes: Vec<Change> = object
            .added
            .into_iter()
            .map(|name| Change {
                path: name.to_owned(),
                kind: ChangeKind::AddedObjectType,
            })
            .collect();

        changes.sort();

        changes
    }
}

pub fn diff(source: &str, target: &str) -> Result<Vec<Change>, async_graphql_parser::Error> {
    let source = async_graphql_parser::parse_schema(source)?;
    let target = async_graphql_parser::parse_schema(target)?;
    let mut state = DiffState {
        source: &source,
        target: &target,
        definitions: Default::default(),
        fields: Default::default(),
        enum_variants: Default::default(),
        union_members: Default::default(),
        arguments: Default::default(),
    };

    let schema_size_approx = source.definitions.len().max(target.definitions.len());

    let mut types_map: DiffMap<&str, DefinitionKind> = HashMap::with_capacity(schema_size_approx);
    let mut fields_map: DiffMap<(&str, &str), Option<&ast::Type>> = HashMap::with_capacity(schema_size_approx);
    let mut arguments_map: DiffMap<(&str, &str, &str), Option<&ast::Type>> = HashMap::with_capacity(schema_size_approx);

    for (idx, tpe) in source.definitions.iter().enumerate() {
        match tpe {
            async_graphql_parser::types::TypeSystemDefinition::Schema(_) => todo!(),
            async_graphql_parser::types::TypeSystemDefinition::Type(tpe) => {
                let type_name = tpe.node.name.node.as_str();

                match &tpe.node.kind {
                    ast::TypeKind::Scalar => {
                        types_map.insert(type_name, (Some(DefinitionKind::Scalar), None));
                    }
                    ast::TypeKind::Object(obj) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Object), None));

                        for field in &obj.fields {
                            insert_source(
                                &mut fields_map,
                                (type_name, field.node.name.node.as_str()),
                                Some(&field.node.ty.node),
                            );
                        }
                    }
                    ast::TypeKind::Interface(iface) => {
                        types_map.insert(type_name, (Some(DefinitionKind::Interface), None));

                        for field in &iface.fields {
                            insert_source(
                                &mut fields_map,
                                (type_name, field.node.name.node.as_str()),
                                Some(&field.node.ty.node),
                            );
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
                                &mut arguments_map,
                                (type_name, field.node.name.node.as_str(), "input"),
                                Some(&field.node.ty.node),
                            );
                        }
                    }
                }
            }
            async_graphql_parser::types::TypeSystemDefinition::Directive(_) => todo!(),
        }
    }

    for (idx, tpe) in target.definitions.iter().enumerate() {
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
                            merge_target(
                                fields_map.entry((type_name, field.node.name.node.as_str())),
                                Some(&field.node.ty.node),
                            );
                        }
                    }
                    ast::TypeKind::Interface(iface) => {
                        types_map.entry(type_name).or_default().1 = Some(DefinitionKind::Interface);

                        for field in &iface.fields {
                            merge_target(
                                fields_map.entry((type_name, field.node.name.node.as_str())),
                                Some(&field.node.ty.node),
                            );
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
                                arguments_map.entry((type_name, field.node.name.node.as_str(), "input")),
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

    for (path @ (type_name, field_name), presence) in fields_map {}

    Ok(state.into_changes())
}
