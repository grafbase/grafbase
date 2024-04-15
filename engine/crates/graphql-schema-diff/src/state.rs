use crate::*;

pub(crate) type DiffMap<K, V> = HashMap<K, (Option<V>, Option<V>)>;

#[derive(Default)]
pub(crate) struct DiffState<'a> {
    pub(crate) schema_definition_map: [Option<ast::SchemaDefinition<'a>>; 2],
    pub(crate) types_map: DiffMap<&'a str, DefinitionKind>,
    pub(crate) fields_map: DiffMap<[&'a str; 2], Option<ast::Type<'a>>>,
    pub(crate) interface_impls: DiffMap<&'a str, Vec<&'a str>>,
    pub(crate) arguments_map: DiffMap<[&'a str; 3], (ast::Type<'a>, Option<ast::Value<'a>>)>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub(crate) enum DefinitionKind {
    Directive,
    Enum,
    InputObject,
    Interface,
    Object,
    Scalar,
    Union,
}

impl DiffState<'_> {
    pub(crate) fn into_changes(self) -> Vec<Change> {
        let DiffState {
            schema_definition_map,
            types_map,
            fields_map,
            arguments_map,
            interface_impls,
        } = self;

        let mut changes = Vec::new();

        push_schema_definition_changes(schema_definition_map, &mut changes);
        push_interface_implementer_changes(interface_impls, &mut changes);

        push_definition_changes(&types_map, &mut changes);
        push_field_changes(&fields_map, &types_map, &mut changes);
        push_argument_changes(&fields_map, &arguments_map, &mut changes);

        changes.sort();

        changes
    }
}

fn push_interface_implementer_changes(interface_impls: DiffMap<&str, Vec<&str>>, changes: &mut Vec<Change>) {
    // O(n²) but n should always be small enough to not matter
    for (implementer, (src, target)) in &interface_impls {
        let src = src.as_deref().unwrap_or(&[]);
        let target = target.as_deref().unwrap_or(&[]);

        for src_impl in src {
            if !target.contains(src_impl) {
                changes.push(Change {
                    path: format!("{}.{}", src_impl, implementer),
                    kind: ChangeKind::RemoveInterfaceImplementation,
                });
            }
        }

        for target_impl in target {
            if !src.contains(target_impl) {
                changes.push(Change {
                    path: format!("{}.{}", target_impl, implementer),
                    kind: ChangeKind::AddInterfaceImplementation,
                });
            }
        }
    }
}

fn push_argument_changes(
    fields_map: &DiffMap<[&str; 2], Option<ast::Type<'_>>>,
    arguments_map: &DiffMap<[&str; 3], (ast::Type<'_>, Option<ast::Value<'_>>)>,
    changes: &mut Vec<Change>,
) {
    for (path @ [type_name, field_name, _arg_name], (src, target)) in arguments_map {
        let path = *path;
        let parent_is_gone = || matches!(&fields_map[&[*type_name, *field_name]], (Some(_), None));

        let kind = match (src, target) {
            (None, None) => unreachable!(),
            (None, Some(_)) => Some(ChangeKind::AddFieldArgument),
            (Some(_), None) if !parent_is_gone() => Some(ChangeKind::RemoveFieldArgument),
            (Some(_), None) => None,
            (Some((src_type, src_default)), Some((target_type, target_default))) => {
                if !type_eq(src_type, target_type) {
                    changes.push(Change {
                        path: path.join("."),
                        kind: ChangeKind::ChangeFieldArgumentType,
                    });
                }

                match (src_default, target_default) {
                    (None, Some(_)) => Some(ChangeKind::AddFieldArgumentDefault),
                    (Some(_), None) => Some(ChangeKind::RemoveFieldArgumentDefault),
                    (Some(a), Some(b)) if !value_eq(a, b) => Some(ChangeKind::ChangeFieldArgumentDefault),
                    _ => None,
                }
            }
        };

        if let Some(kind) = kind {
            changes.push(Change {
                path: path.join("."),
                kind,
            });
        }
    }
}

fn push_field_changes(
    fields_map: &DiffMap<[&str; 2], Option<ast::Type<'_>>>,
    types_map: &DiffMap<&str, DefinitionKind>,
    changes: &mut Vec<Change>,
) {
    for (path @ [type_name, _field_name], (src, target)) in fields_map {
        let parent = &types_map[type_name];
        let parent_is_gone = || matches!(parent, (Some(_), None));

        let definition = match parent {
            (None, None) => unreachable!(),
            (Some(a), Some(b)) if a != b => {
                continue; // so we don't falsely interpret same name as field type change
            }
            (Some(_), None) | (None, Some(_)) => continue,
            (Some(kind), Some(_)) => *kind,
        };

        let change_kind = match (src, target, definition) {
            (None, None, _) | (_, _, DefinitionKind::Scalar | DefinitionKind::Directive) => {
                unreachable!()
            }
            (None, Some(_), DefinitionKind::Object | DefinitionKind::Interface | DefinitionKind::InputObject) => {
                Some(ChangeKind::AddField)
            }
            (None, Some(_), DefinitionKind::Enum) => Some(ChangeKind::AddEnumValue),
            (Some(_), None, DefinitionKind::Enum) if !parent_is_gone() => Some(ChangeKind::RemoveEnumValue),
            (None, Some(_), DefinitionKind::Union) => Some(ChangeKind::AddUnionMember),
            (Some(_), None, DefinitionKind::Union) if !parent_is_gone() => Some(ChangeKind::RemoveUnionMember),
            (Some(_), None, DefinitionKind::Object | DefinitionKind::Interface | DefinitionKind::InputObject)
                if !parent_is_gone() =>
            {
                Some(ChangeKind::RemoveField)
            }
            (
                Some(ty_a),
                Some(ty_b),
                DefinitionKind::Object | DefinitionKind::InputObject | DefinitionKind::Interface,
            ) if !opt_type_eq(ty_a.as_ref(), ty_b.as_ref()) => Some(ChangeKind::ChangeFieldType),
            (Some(_), None, _) => None,
            (Some(_), Some(_), _) => None,
        };

        if let Some(kind) = change_kind {
            changes.push(Change {
                path: path.join("."),
                kind,
            });
        }
    }
}

fn push_definition_changes(
    types_map: &HashMap<&str, (Option<DefinitionKind>, Option<DefinitionKind>)>,
    changes: &mut Vec<Change>,
) {
    for (name, entries) in types_map {
        match entries {
            (None, None) => unreachable!(),
            (None, Some(definition)) => push_added_type(name, *definition, changes),
            (Some(definition), None) => push_removed_type(name, *definition, changes),
            (Some(a), Some(b)) if a != b => {
                push_removed_type(name, *a, changes);
                push_added_type(name, *b, changes);
            }
            (Some(_), Some(_)) => (),
        }
    }
}

fn push_added_type(name: &str, kind: DefinitionKind, changes: &mut Vec<Change>) {
    let change_kind = match kind {
        DefinitionKind::Directive => ChangeKind::AddDirectiveDefinition,
        DefinitionKind::Enum => ChangeKind::AddEnum,
        DefinitionKind::InputObject => ChangeKind::AddInputObject,
        DefinitionKind::Interface => ChangeKind::AddInterface,
        DefinitionKind::Object => ChangeKind::AddObjectType,
        DefinitionKind::Scalar => ChangeKind::AddScalar,
        DefinitionKind::Union => ChangeKind::AddUnion,
    };

    changes.push(Change {
        path: name.to_owned(),
        kind: change_kind,
    });
}

fn push_removed_type(name: &str, kind: DefinitionKind, changes: &mut Vec<Change>) {
    let change_kind = match kind {
        DefinitionKind::Directive => ChangeKind::RemoveDirectiveDefinition,
        DefinitionKind::Enum => ChangeKind::RemoveEnum,
        DefinitionKind::InputObject => ChangeKind::RemoveInputObject,
        DefinitionKind::Interface => ChangeKind::RemoveInterface,
        DefinitionKind::Object => ChangeKind::RemoveObjectType,
        DefinitionKind::Scalar => ChangeKind::RemoveScalar,
        DefinitionKind::Union => ChangeKind::RemoveUnion,
    };

    changes.push(Change {
        path: name.to_owned(),
        kind: change_kind,
    });
}

fn push_schema_definition_changes(
    schema_definition_map: [Option<ast::SchemaDefinition<'_>>; 2],
    changes: &mut Vec<Change>,
) {
    match schema_definition_map {
        [None, None] => (),
        [Some(src), Some(target)] => {
            let [src_query, src_mutation, src_subscription] =
                [src.query_type(), src.mutation_type(), src.subscription_type()];

            let [target_query, target_mutation, target_subscription] =
                [target.query_type(), target.mutation_type(), target.subscription_type()];

            if src_query != target_query {
                changes.push(Change {
                    path: String::new(),
                    kind: ChangeKind::ChangeQueryType,
                });
            }

            if src_mutation != target_mutation {
                changes.push(Change {
                    path: String::new(),
                    kind: ChangeKind::ChangeMutationType,
                });
            }

            if src_subscription != target_subscription {
                changes.push(Change {
                    path: String::new(),
                    kind: ChangeKind::ChangeSubscriptionType,
                });
            }
        }
        [None, Some(_)] => changes.push(Change {
            path: String::new(),
            kind: ChangeKind::AddSchemaDefinition,
        }),
        [Some(_), None] => changes.push(Change {
            path: String::new(),
            kind: ChangeKind::RemoveSchemaDefinition,
        }),
    }
}

fn opt_type_eq(a: Option<&ast::Type<'_>>, b: Option<&ast::Type<'_>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => type_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

fn type_eq(a: &ast::Type<'_>, b: &ast::Type<'_>) -> bool {
    a.name() == b.name() && a.wrappers().eq(b.wrappers())
}

fn value_eq(a: &ast::Value<'_>, b: &ast::Value<'_>) -> bool {
    // Same enum variants and equal values
    match (a, b) {
        (ast::Value::Int(a), ast::Value::Int(b)) => a == b,
        (ast::Value::Variable(a), ast::Value::Variable(b)) => a == b,
        (ast::Value::Float(a), ast::Value::Float(b)) => a == b,
        (ast::Value::String(a), ast::Value::BlockString(b)) => a == b,
        (ast::Value::String(a), ast::Value::String(b)) => a == b,
        (ast::Value::BlockString(a), ast::Value::String(b)) => a == b,
        (ast::Value::BlockString(a), ast::Value::BlockString(b)) => a == b,
        (ast::Value::Boolean(a), ast::Value::Boolean(b)) => a == b,
        (ast::Value::Null, ast::Value::Null) => true,
        (ast::Value::Enum(a), ast::Value::Enum(b)) => a == b,
        (ast::Value::List(a), ast::Value::List(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| value_eq(a, b))
        }
        (ast::Value::Object(a), ast::Value::Object(b)) => {
            a.len() == b.len()
                && a.iter().all(|(name, value)| {
                    let Some((_, b_value)) = b.iter().find(|(b_name, _)| b_name == name) else {
                        return false;
                    };

                    value_eq(value, b_value)
                })
        }
        _ => false,
    }
}
