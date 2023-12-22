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

        push_schema_definition_changes(schema_definition_map, &mut changes);

        push_definition_changes(types_map, &mut changes);

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

fn push_definition_changes(
    types_map: HashMap<&str, (Option<DefinitionKind>, Option<DefinitionKind>)>,
    changes: &mut Vec<Change>,
) {
    for (name, entries) in &types_map {
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
