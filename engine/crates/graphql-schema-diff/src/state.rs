use change::Span;

use crate::*;

pub(crate) type DiffMap<K, V> = HashMap<K, (Option<V>, Option<V>)>;

#[derive(Default)]
pub(crate) struct DiffState<'a> {
    pub(crate) schema_definition_map: [Option<ast::SchemaDefinition<'a>>; 2],
    pub(crate) types_map: DiffMap<&'a str, ast::Definition<'a>>,
    pub(crate) fields_map: DiffMap<[&'a str; 2], (Option<ast::Type<'a>>, Span)>,
    pub(crate) interface_impls: DiffMap<&'a str, Vec<&'a str>>,
    pub(crate) arguments_map: DiffMap<[&'a str; 3], ast::InputValueDefinition<'a>>,
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

impl DefinitionKind {
    pub(crate) fn new(definition: &ast::Definition<'_>) -> Option<Self> {
        match definition {
            ast::Definition::Schema(_) | ast::Definition::SchemaExtension(_) => None,
            ast::Definition::Type(ty) | ast::Definition::TypeExtension(ty) => match ty {
                ast::TypeDefinition::Scalar(_) => Some(DefinitionKind::Scalar),
                ast::TypeDefinition::Object(_) => Some(DefinitionKind::Object),
                ast::TypeDefinition::Interface(_) => Some(DefinitionKind::Interface),
                ast::TypeDefinition::Union(_) => Some(DefinitionKind::Union),
                ast::TypeDefinition::Enum(_) => Some(DefinitionKind::Enum),
                ast::TypeDefinition::InputObject(_) => Some(DefinitionKind::InputObject),
            },
            ast::Definition::Directive(_) => Some(DefinitionKind::Directive),
        }
    }
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
                    span: Span::empty(),
                });
            }
        }

        for target_impl in target {
            if !src.contains(target_impl) {
                changes.push(Change {
                    path: format!("{}.{}", target_impl, implementer),
                    kind: ChangeKind::AddInterfaceImplementation,
                    span: Span::empty(),
                });
            }
        }
    }
}

fn push_argument_changes(
    fields_map: &DiffMap<[&str; 2], (Option<ast::Type<'_>>, Span)>,
    arguments_map: &DiffMap<[&str; 3], ast::InputValueDefinition<'_>>,
    changes: &mut Vec<Change>,
) {
    for (path @ [type_name, field_name, _arg_name], (src, target)) in arguments_map {
        let path = *path;
        let parent_is_gone = || matches!(&fields_map[&[*type_name, *field_name]], (Some(_), None));

        match (src, target) {
            (None, None) => unreachable!(),
            (None, Some(target)) => {
                changes.push(Change {
                    path: path.join("."),
                    kind: ChangeKind::AddFieldArgument,
                    span: target.span().into(),
                });
            }
            (Some(_), None) if !parent_is_gone() => {
                changes.push(Change {
                    path: path.join("."),
                    kind: ChangeKind::RemoveFieldArgument,
                    span: Span::empty(),
                });
            }
            (Some(_), None) => (),
            (Some(src_arg), Some(target_arg)) => {
                if src_arg.ty() != target_arg.ty() {
                    changes.push(Change {
                        path: path.join("."),
                        kind: ChangeKind::ChangeFieldArgumentType,
                        span: target_arg.ty().span().into(),
                    });
                }

                match (src_arg.default_value(), target_arg.default_value()) {
                    (None, Some(_)) => changes.push(Change {
                        path: path.join("."),
                        kind: ChangeKind::AddFieldArgumentDefault,
                        span: target_arg.default_value_span().into(),
                    }),
                    (Some(_), None) => changes.push(Change {
                        path: path.join("."),
                        kind: ChangeKind::RemoveFieldArgumentDefault,
                        span: Span::empty(),
                    }),
                    (Some(a), Some(b)) if a != b => changes.push(Change {
                        path: path.join("."),
                        kind: ChangeKind::ChangeFieldArgumentDefault,
                        span: target_arg.default_value_span().into(),
                    }),
                    _ => (),
                }
            }
        };
    }
}

fn push_field_changes(
    fields_map: &DiffMap<[&str; 2], (Option<ast::Type<'_>>, Span)>,
    types_map: &DiffMap<&str, ast::Definition<'_>>,
    changes: &mut Vec<Change>,
) {
    for (path @ [type_name, _field_name], (src, target)) in fields_map {
        let parent = &types_map[type_name];
        let parent_is_gone = || matches!(parent, (Some(_), None));

        let definition = match parent {
            (None, None) => unreachable!(),
            (Some(a), Some(b)) if DefinitionKind::new(a) != DefinitionKind::new(b) => {
                continue; // so we don't falsely interpret same name as field type change
            }
            (Some(_), None) | (None, Some(_)) => continue,
            (Some(kind), Some(_)) => *kind,
        };

        let change_kind = match (src, target, DefinitionKind::new(&definition).unwrap()) {
            (None, None, _) | (_, _, DefinitionKind::Scalar | DefinitionKind::Directive) => {
                unreachable!()
            }
            (
                None,
                Some((_, span)),
                DefinitionKind::Object | DefinitionKind::Interface | DefinitionKind::InputObject,
            ) => Some((ChangeKind::AddField, *span)),
            (None, Some((_, span)), DefinitionKind::Enum) => Some((ChangeKind::AddEnumValue, *span)),
            (Some(_), None, DefinitionKind::Enum) if !parent_is_gone() => {
                Some((ChangeKind::RemoveEnumValue, Span::empty()))
            }
            (None, Some((_, span)), DefinitionKind::Union) => Some((ChangeKind::AddUnionMember, *span)),
            (Some(_), None, DefinitionKind::Union) if !parent_is_gone() => {
                Some((ChangeKind::RemoveUnionMember, Span::empty()))
            }
            (Some(_), None, DefinitionKind::Object | DefinitionKind::Interface | DefinitionKind::InputObject)
                if !parent_is_gone() =>
            {
                Some((ChangeKind::RemoveField, Span::empty()))
            }
            (
                Some((ty_a, _)),
                Some((ty_b, _)),
                DefinitionKind::Object | DefinitionKind::InputObject | DefinitionKind::Interface,
            ) if ty_a.as_ref() != ty_b.as_ref() => Some((ChangeKind::ChangeFieldType, ty_b.unwrap().span().into())),
            (Some(_), None, _) => None,
            (Some(_), Some(_), _) => None,
        };

        if let Some((kind, span)) = change_kind {
            changes.push(Change {
                path: path.join("."),
                kind,
                span,
            });
        }
    }
}

fn push_definition_changes(
    types_map: &HashMap<&str, (Option<ast::Definition<'_>>, Option<ast::Definition<'_>>)>,
    changes: &mut Vec<Change>,
) {
    for (name, entries) in types_map {
        match entries {
            (None, None) => unreachable!(),
            (None, Some(definition)) => push_added_type(name, *definition, changes),
            (Some(definition), None) => push_removed_type(name, *definition, changes),
            (Some(a), Some(b)) if DefinitionKind::new(a) != DefinitionKind::new(b) => {
                push_removed_type(name, *a, changes);
                push_added_type(name, *b, changes);
            }
            (Some(_), Some(_)) => (),
        }
    }
}

fn push_added_type(name: &str, definition: ast::Definition<'_>, changes: &mut Vec<Change>) {
    let Some(kind) = DefinitionKind::new(&definition) else {
        return;
    };

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
        span: definition.span().into(),
    });
}

fn push_removed_type(name: &str, definition: ast::Definition<'_>, changes: &mut Vec<Change>) {
    let change_kind = match DefinitionKind::new(&definition).unwrap() {
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
        span: definition.span().into(),
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

            if src_query.map(|ty| ty.named_type()) != target_query.map(|ty| ty.named_type()) {
                changes.push(Change {
                    path: String::new(),
                    kind: ChangeKind::ChangeQueryType,
                    span: target_query.map(|ty| ty.span().into()).unwrap_or_else(Span::empty),
                });
            }

            if src_mutation.map(|ty| ty.named_type()) != target_mutation.map(|ty| ty.named_type()) {
                changes.push(Change {
                    path: String::new(),
                    kind: ChangeKind::ChangeMutationType,
                    span: target_mutation.map(|ty| ty.span().into()).unwrap_or_else(Span::empty),
                });
            }

            if src_subscription.map(|ty| ty.named_type()) != target_subscription.map(|ty| ty.named_type()) {
                changes.push(Change {
                    path: String::new(),
                    kind: ChangeKind::ChangeSubscriptionType,
                    span: target_subscription
                        .map(|ty| ty.span().into())
                        .unwrap_or_else(Span::empty),
                });
            }
        }
        [None, Some(definition)] => changes.push(Change {
            path: String::new(),
            kind: ChangeKind::AddSchemaDefinition,
            span: definition.span().into(),
        }),
        [Some(_), None] => changes.push(Change {
            path: String::new(),
            kind: ChangeKind::RemoveSchemaDefinition,
            span: Span::empty(),
        }),
    }
}
