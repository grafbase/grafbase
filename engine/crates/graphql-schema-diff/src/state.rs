use crate::{Change, ChangeKind};
use async_graphql_parser::types as ast;
use std::collections::HashMap;

pub(crate) type DiffMap<K, V> = HashMap<K, (Option<V>, Option<V>)>;

#[derive(Debug, Default)]
pub(crate) struct AddedRemoved<T> {
    pub(crate) added: T,
    pub(crate) removed: T,
}

#[derive(Default)]
pub(crate) struct DiffState<'a> {
    pub(crate) definitions: Definitions<'a>,
    pub(crate) fields: AddedRemoved<Vec<[&'a str; 2]>>,
    pub(crate) enum_variants: AddedRemoved<Vec<[&'a str; 2]>>,
    pub(crate) union_members: AddedRemoved<Vec<[&'a str; 2]>>,
    pub(crate) arguments: AddedRemoved<Vec<[&'a str; 3]>>,
    pub(crate) argument_default_values: AddedRemoved<Vec<[&'a str; 3]>>,
    pub(crate) argument_default_changed: Vec<[&'a str; 3]>,
    pub(crate) argument_type_changed: Vec<[&'a str; 3]>,
    pub(crate) field_type_changed: Vec<[&'a str; 2]>,
    pub(crate) schema_definition_map: (Option<&'a ast::SchemaDefinition>, Option<&'a ast::SchemaDefinition>),
}

macro_rules! definition_kinds {
    ($($camel:ident, $snake:ident);*) => {
            #[derive(Debug, PartialEq, Eq, Clone, Copy)]
            #[repr(u8)]
            pub(crate) enum DefinitionKind {
                $(
                    $camel,
                )*
            }

            #[derive(Default)]
            pub(crate) struct Definitions<'a> {
                $(
                    pub(crate) $snake: AddedRemoved<Vec<&'a str>>,
                )*
            }

            impl<'a> DiffState<'a> {
                pub(crate) fn push_added_type(&mut self, name: &'a str, kind: DefinitionKind) {
                    match kind {
                        $(
                            DefinitionKind::$camel => self.definitions.$snake.added.push(name),
                        )*
                    }
                }

                pub(crate) fn push_removed_type(&mut self, name: &'a str, kind: DefinitionKind) {
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
    Union, union
}

impl DiffState<'_> {
    pub(crate) fn into_changes(self) -> Vec<Change> {
        let DiffState {
            definitions:
                Definitions {
                    directive,
                    r#enum,
                    input_object,
                    interface,
                    object,
                    scalar,
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
            schema_definition_map,
        } = self;

        let mut changes = Vec::new();

        match schema_definition_map {
            (None, None) | (Some(_), Some(_)) => (),
            (None, Some(_)) => changes.push(Change {
                path: String::new(),
                kind: ChangeKind::AddSchemaDefinition,
            }),
            (Some(_), None) => changes.push(Change {
                path: String::new(),
                kind: ChangeKind::RemoveSchemaDefinition,
            }),
        }

        changes.extend(
            [
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
                (input_object.added, ChangeKind::AddInputObject),
                (input_object.removed, ChangeKind::RemoveInputObject),
            ]
            .into_iter()
            .flat_map(|(items, kind)| {
                items.into_iter().map(move |name| Change {
                    path: name.to_owned(),
                    kind,
                })
            }),
        );

        changes.extend(
            [
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
            }),
        );

        changes.extend(
            [
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
            }),
        );

        changes.sort();

        changes
    }
}
