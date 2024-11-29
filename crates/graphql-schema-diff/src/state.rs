use change::Span;
use path::PathInType;

use crate::*;

pub(crate) type DiffMap<K, V> = HashMap<K, (Option<V>, Option<V>)>;

#[derive(Default)]
pub(crate) struct DiffState<'a> {
    pub(crate) schema_definition_map: [Option<ast::SchemaDefinition<'a>>; 2],
    pub(crate) schema_extensions: Vec<[Option<ast::SchemaDefinition<'a>>; 2]>,
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
    pub(crate) fn into_changes(self, config: &DiffConfig) -> Vec<Change> {
        let DiffState {
            schema_definition_map,
            schema_extensions,
            types_map,
            fields_map,
            arguments_map,
            interface_impls,
        } = self;

        let mut changes = Vec::new();
        let mut push_change = |path: path::Path<'_>, kind: ChangeKind, span: Span| {
            changes.push(Change {
                path: path.to_string(),
                kind,
                span,
            })
        };

        push_schema_definition_changes(schema_definition_map, &mut push_change);
        push_schema_extension_changes(schema_extensions, &mut push_change);
        push_interface_implementer_changes(interface_impls, &mut push_change);

        push_definition_changes(&types_map, &mut push_change);
        push_field_changes(&fields_map, &types_map, &mut push_change, config);
        push_argument_changes(&fields_map, &arguments_map, &mut push_change);

        changes.sort();

        changes
    }
}

type PushChangeFn<'a> = &'a mut dyn FnMut(path::Path<'_>, ChangeKind, Span);

fn push_schema_extension_changes(
    schema_extensions: Vec<[Option<ast::SchemaDefinition<'_>>; 2]>,
    push_change: PushChangeFn<'_>,
) {
    for extension in schema_extensions {
        match extension {
            // TODO(GB-7390): we only react to additions and removals for now. We should do full diffing.
            [None, Some(def)] => push_change(
                path::Path::SchemaExtension(0),
                ChangeKind::AddSchemaExtension,
                def.span().into(),
            ),
            [Some(def), None] => push_change(
                path::Path::SchemaExtension(0),
                ChangeKind::RemoveSchemaExtension,
                def.span().into(),
            ),
            _ => (),
        }
    }
}

fn push_interface_implementer_changes(interface_impls: DiffMap<&str, Vec<&str>>, push_change: PushChangeFn<'_>) {
    // O(n²) but n should always be small enough to not matter
    for (implementer, (src, target)) in &interface_impls {
        let src = src.as_deref().unwrap_or(&[]);
        let target = target.as_deref().unwrap_or(&[]);

        for src_impl in src {
            if !target.contains(src_impl) {
                push_change(
                    path::Path::TypeDefinition(implementer, Some(PathInType::InterfaceImplementation(src_impl))),
                    ChangeKind::RemoveInterfaceImplementation,
                    Span::empty(),
                );
            }
        }

        for target_impl in target {
            if !src.contains(target_impl) {
                push_change(
                    path::Path::TypeDefinition(implementer, Some(PathInType::InterfaceImplementation(target_impl))),
                    ChangeKind::AddInterfaceImplementation,
                    Span::empty(),
                );
            }
        }
    }
}

fn push_argument_changes(
    fields_map: &DiffMap<[&str; 2], (Option<ast::Type<'_>>, Span)>,
    arguments_map: &DiffMap<[&str; 3], ast::InputValueDefinition<'_>>,
    push_change: PushChangeFn<'_>,
) {
    for ([type_name, field_name, arg_name], (src, target)) in arguments_map {
        let parent_is_gone = || matches!(&fields_map[&[*type_name, *field_name]], (Some(_), None));

        let argument_path = path::Path::TypeDefinition(
            type_name,
            Some(path::PathInType::InField(
                field_name,
                Some(path::PathInField::InArgument(arg_name)),
            )),
        );

        match (src, target) {
            (None, None) => unreachable!(),
            (None, Some(target)) => {
                push_change(argument_path, ChangeKind::AddFieldArgument, target.span().into());
            }
            (Some(_), None) if !parent_is_gone() => {
                push_change(argument_path, ChangeKind::RemoveFieldArgument, Span::empty());
            }
            (Some(_), None) => (),
            (Some(src_arg), Some(target_arg)) => {
                if src_arg.ty() != target_arg.ty() {
                    push_change(
                        argument_path.clone(),
                        ChangeKind::ChangeFieldArgumentType,
                        target_arg.ty().span().into(),
                    );
                }

                match (src_arg.default_value(), target_arg.default_value()) {
                    (None, Some(_)) => push_change(
                        argument_path,
                        ChangeKind::AddFieldArgumentDefault,
                        target_arg.default_value_span().into(),
                    ),
                    (Some(_), None) => {
                        push_change(argument_path, ChangeKind::RemoveFieldArgumentDefault, Span::empty())
                    }
                    (Some(a), Some(b)) if a != b => push_change(
                        argument_path,
                        ChangeKind::ChangeFieldArgumentDefault,
                        target_arg.default_value_span().into(),
                    ),
                    _ => (),
                }
            }
        };
    }
}

fn push_field_changes(
    fields_map: &DiffMap<[&str; 2], (Option<ast::Type<'_>>, Span)>,
    types_map: &DiffMap<&str, ast::Definition<'_>>,
    push_change: PushChangeFn<'_>,
    config: &DiffConfig,
) {
    for ([type_name, field_name], (src, target)) in fields_map {
        let parent = &types_map[type_name];
        let parent_is_gone = || matches!(parent, (Some(_), None));

        let definition = match parent {
            (None, None) => unreachable!(),
            (Some(a), Some(b)) if DefinitionKind::new(a) != DefinitionKind::new(b) => {
                continue; // so we don't falsely interpret same name as field type change
            }
            (Some(_), None) => continue,
            (None, Some(definition)) => {
                if config.additions_inside_type_definitions {
                    *definition
                } else {
                    continue;
                }
            }
            (Some(definition), Some(_)) => *definition,
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
            push_change(
                path::Path::TypeDefinition(type_name, Some(path::PathInType::InField(field_name, None))),
                kind,
                span,
            )
        }
    }
}

fn push_definition_changes(
    types_map: &HashMap<&str, (Option<ast::Definition<'_>>, Option<ast::Definition<'_>>)>,
    push_change: PushChangeFn<'_>,
) {
    for (name, entries) in types_map {
        match entries {
            (None, None) => unreachable!(),
            (None, Some(definition)) => push_added_type(name, *definition, push_change),
            (Some(definition), None) => push_removed_type(name, *definition, push_change),
            (Some(a), Some(b)) if DefinitionKind::new(a) != DefinitionKind::new(b) => {
                push_removed_type(name, *a, push_change);
                push_added_type(name, *b, push_change);
            }
            (Some(_), Some(_)) => (),
        }
    }
}

fn push_added_type(name: &str, definition: ast::Definition<'_>, push_change: PushChangeFn<'_>) {
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

    push_change(
        path::Path::TypeDefinition(name, None),
        change_kind,
        definition.span().into(),
    )
}

fn push_removed_type(name: &str, definition: ast::Definition<'_>, push_change: PushChangeFn<'_>) {
    let change_kind = match DefinitionKind::new(&definition).unwrap() {
        DefinitionKind::Directive => ChangeKind::RemoveDirectiveDefinition,
        DefinitionKind::Enum => ChangeKind::RemoveEnum,
        DefinitionKind::InputObject => ChangeKind::RemoveInputObject,
        DefinitionKind::Interface => ChangeKind::RemoveInterface,
        DefinitionKind::Object => ChangeKind::RemoveObjectType,
        DefinitionKind::Scalar => ChangeKind::RemoveScalar,
        DefinitionKind::Union => ChangeKind::RemoveUnion,
    };

    push_change(
        path::Path::TypeDefinition(name, None),
        change_kind,
        definition.span().into(),
    );
}

fn push_schema_definition_changes(
    schema_definition_map: [Option<ast::SchemaDefinition<'_>>; 2],
    push_change: PushChangeFn<'_>,
) {
    match schema_definition_map {
        [None, None] => (),
        [Some(src), Some(target)] => {
            let [src_query, src_mutation, src_subscription] =
                [src.query_type(), src.mutation_type(), src.subscription_type()];

            let [target_query, target_mutation, target_subscription] =
                [target.query_type(), target.mutation_type(), target.subscription_type()];

            if src_query.map(|ty| ty.named_type()) != target_query.map(|ty| ty.named_type()) {
                push_change(
                    path::Path::SchemaDefinition,
                    ChangeKind::ChangeQueryType,
                    target_query
                        .map(|ty| ty.named_type_span().into())
                        .unwrap_or_else(Span::empty),
                );
            }

            if src_mutation.map(|ty| ty.named_type()) != target_mutation.map(|ty| ty.named_type()) {
                push_change(
                    path::Path::SchemaDefinition,
                    ChangeKind::ChangeMutationType,
                    target_mutation
                        .map(|ty| ty.named_type_span().into())
                        .unwrap_or_else(Span::empty),
                );
            }

            if src_subscription.map(|ty| ty.named_type()) != target_subscription.map(|ty| ty.named_type()) {
                push_change(
                    path::Path::SchemaDefinition,
                    ChangeKind::ChangeSubscriptionType,
                    target_subscription
                        .map(|ty| ty.named_type_span().into())
                        .unwrap_or_else(Span::empty),
                );
            }
        }
        [None, Some(definition)] => push_change(
            path::Path::SchemaDefinition,
            ChangeKind::AddSchemaDefinition,
            definition.span().into(),
        ),
        [Some(_), None] => push_change(
            path::Path::SchemaDefinition,
            ChangeKind::RemoveSchemaDefinition,
            Span::empty(),
        ),
    }
}
