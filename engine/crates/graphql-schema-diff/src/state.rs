use crate::*;

pub(crate) type DiffMap<K, V> = HashMap<K, (Option<V>, Option<V>)>;

#[derive(Debug, Default)]
pub(crate) struct AddedRemoved<T> {
    pub(crate) added: T,
    pub(crate) removed: T,
}

#[derive(Default)]
pub(crate) struct DiffState<'a> {
    pub(crate) fields: AddedRemoved<Vec<[&'a str; 2]>>,
    pub(crate) enum_variants: AddedRemoved<Vec<[&'a str; 2]>>,
    pub(crate) union_members: AddedRemoved<Vec<[&'a str; 2]>>,
    pub(crate) arguments: AddedRemoved<Vec<[&'a str; 3]>>,
    pub(crate) argument_default_values: AddedRemoved<Vec<[&'a str; 3]>>,
    pub(crate) argument_default_changed: Vec<[&'a str; 3]>,
    pub(crate) argument_type_changed: Vec<[&'a str; 3]>,
    pub(crate) field_type_changed: Vec<[&'a str; 2]>,

    pub(crate) schema_definition_map: (Option<&'a ast::SchemaDefinition>, Option<&'a ast::SchemaDefinition>),
    pub(crate) types_map: DiffMap<&'a str, DefinitionKind>,
    pub(crate) fields_map: DiffMap<[&'a str; 2], Option<&'a ast::Type>>,
    pub(crate) arguments_map: DiffMap<[&'a str; 3], (&'a ast::Type, Option<&'a ConstValue>)>,
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
            fields,
            enum_variants,
            union_members,
            arguments,
            argument_default_values,
            argument_default_changed,
            field_type_changed,
            argument_type_changed,

            schema_definition_map,
            types_map,
            fields_map,
            arguments_map,
        } = self;

        let mut changes = Vec::new();

        // TODO interface implementers
        push_schema_definition_changes(schema_definition_map, &mut changes);

        push_definition_changes(&types_map, &mut changes);

        push_field_changes(&fields_map, &types_map, &mut changes);

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

fn push_field_changes(
    fields_map: &DiffMap<[&str; 2], Option<&ast::Type>>,
    types_map: &DiffMap<&str, DefinitionKind>,
    changes: &mut Vec<Change>,
) {
    for (path @ [type_name, _field_name], (src, target)) in fields_map {
        let parent = &types_map[type_name];
        let parent_is_gone = || matches!(parent, (Some(_), None));

        if matches!(parent, (Some(a), Some(b)) if a != b) {
            continue; // so we don't falsely interpret same name as field type change
        }

        let definition_kind = match parent {
            (None, None) => unreachable!(),
            (Some(kind), None) | (None, Some(kind)) => *kind,
            (Some(kind), Some(_)) => *kind,
        };

        let change_kind = match (src, target, definition_kind) {
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
            ) if ty_a != ty_b => Some(ChangeKind::ChangeFieldType),
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
            (None, Some(kind)) => push_added_type(name, *kind, changes),
            (Some(kind), None) => push_removed_type(name, *kind, changes),
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
    schema_definition_map: (Option<&ast::SchemaDefinition>, Option<&ast::SchemaDefinition>),
    changes: &mut Vec<Change>,
) {
    match schema_definition_map {
        (None, None) => (),
        (Some(src), Some(target)) => {
            let [src_query, src_mutation, src_subscription] =
                [&src.query, &src.mutation, &src.subscription].map(|opt_node| opt_node.as_ref().map(|n| &n.node));
            let [target_query, target_mutation, target_subscription] =
                [&target.query, &target.mutation, &target.subscription]
                    .map(|opt_node| opt_node.as_ref().map(|n| &n.node));
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
        (None, Some(_)) => changes.push(Change {
            path: String::new(),
            kind: ChangeKind::AddSchemaDefinition,
        }),
        (Some(_), None) => changes.push(Change {
            path: String::new(),
            kind: ChangeKind::RemoveSchemaDefinition,
        }),
    }
}
