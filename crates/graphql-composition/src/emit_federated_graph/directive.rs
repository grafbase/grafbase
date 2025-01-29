use crate::composition_ir as ir;
use graphql_federated_graph::{self as federated, Definition, EntityDefinitionId, Wrapping};

use super::{attach_argument_selection::attach_argument_selection, attach_selection, context::Context};

pub(super) fn transform_arbitray_type_directives(
    ctx: &mut Context<'_>,
    directives: Vec<ir::Directive>,
) -> Vec<federated::Directive> {
    directives
        .into_iter()
        .filter_map(|directive| match directive {
            ir::Directive::JoinType(dir) if dir.key.is_none() => {
                Some(federated::Directive::JoinType(federated::JoinTypeDirective {
                    subgraph_id: dir.subgraph_id,
                    key: None,
                    resolvable: true,
                    is_interface_object: false,
                }))
            }
            dir => transform_common_directive(ctx, dir),
        })
        .collect()
}

pub(super) fn transform_input_value_directives(
    ctx: &mut Context<'_>,
    directives: Vec<ir::Directive>,
) -> Vec<federated::Directive> {
    directives
        .into_iter()
        .filter_map(|directive| match directive {
            ir::Directive::JoinInputField(dir) => {
                Some(federated::Directive::JoinField(federated::JoinFieldDirective {
                    subgraph_id: Some(dir.subgraph_id),
                    requires: None,
                    provides: None,
                    r#type: dir.r#type.map(|ty| ctx.insert_field_type(ctx.subgraphs.walk(ty))),
                    r#override: None,
                    override_label: None,
                }))
            }
            dir => transform_common_directive(ctx, dir),
        })
        .collect()
}

pub(super) fn transform_enum_value_directives(
    ctx: &mut Context<'_>,
    directives: Vec<ir::Directive>,
) -> Vec<federated::Directive> {
    directives
        .into_iter()
        .filter_map(|directive| transform_common_directive(ctx, directive))
        .collect()
}

pub(super) fn transform_type_directives(
    ctx: &mut Context<'_>,
    parent: federated::Definition,
    directives: Vec<ir::Directive>,
) -> Vec<federated::Directive> {
    directives
        .into_iter()
        .filter_map(|directive| match (directive, parent) {
            (ir::Directive::Authorized(dir), Definition::Object(id)) => {
                Some(transform_authorized_entity_directive(ctx, id.into(), dir))
            }
            (ir::Directive::Authorized(dir), Definition::Interface(id)) => {
                Some(transform_authorized_entity_directive(ctx, id.into(), dir))
            }
            (ir::Directive::JoinUnionMember(dir), Definition::Union(_)) => {
                transform_join_union_member_directive(ctx, dir)
            }
            (ir::Directive::JoinType(dir), _) => Some(transform_join_type_directive(ctx, parent, dir)),
            (dir, _) => transform_common_directive(ctx, dir),
        })
        .collect()
}

pub(super) fn transform_field_directives(
    ctx: &mut Context<'_>,
    field_id: federated::FieldId,
    directives: Vec<ir::Directive>,
) -> Vec<federated::Directive> {
    directives
        .into_iter()
        .filter_map(|directive| match directive {
            ir::Directive::JoinField(dir) => Some(transform_join_field_directive(ctx, field_id, dir)),
            ir::Directive::JoinEntityInterfaceField => {
                Some(federated::Directive::JoinField(federated::JoinFieldDirective::default()))
            }
            ir::Directive::Authorized(dir) => Some(transform_authorized_field_directive(ctx, field_id, dir)),
            ir::Directive::ListSize(dir) => Some(transform_list_size_directive(ctx, field_id, dir)),
            dir => transform_common_directive(ctx, dir),
        })
        .collect()
}

fn transform_common_directive(ctx: &mut Context<'_>, directive: ir::Directive) -> Option<federated::Directive> {
    Some(match directive {
        ir::Directive::Authenticated => federated::Directive::Authenticated,
        ir::Directive::Deprecated { reason } => federated::Directive::Deprecated { reason },
        ir::Directive::Inaccessible => federated::Directive::Inaccessible,
        ir::Directive::Policy(policies) => federated::Directive::Policy(policies),
        ir::Directive::RequiresScopes(scopes) => federated::Directive::RequiresScopes(scopes),
        ir::Directive::Cost { weight } => {
            ctx.uses_cost_directive = true;
            federated::Directive::Cost { weight }
        }
        ir::Directive::Other { name, arguments } => federated::Directive::Other {
            name,
            arguments: arguments
                .into_iter()
                .map(|(name, value)| (name, ctx.insert_value(&value)))
                .collect(),
        },
        ir::Directive::JoinField(_)
        | ir::Directive::Authorized(_)
        | ir::Directive::JoinType(_)
        | ir::Directive::ListSize(_)
        | ir::Directive::JoinUnionMember(_)
        | ir::Directive::JoinInputField(_)
        | ir::Directive::JoinEntityInterfaceField => {
            return None;
        }
    })
}

fn transform_join_union_member_directive(
    ctx: &mut Context<'_>,
    ir::JoinUnionMemberDirective { member }: ir::JoinUnionMemberDirective,
) -> Option<federated::Directive> {
    let member = ctx.subgraphs.walk(member);
    let name = ctx.insert_string(member.name());
    match &ctx.definitions[&name] {
        Definition::Object(object_id) => Some(federated::Directive::JoinUnionMember(
            federated::JoinUnionMemberDirective {
                subgraph_id: federated::SubgraphId::from(member.subgraph_id().idx()),
                object_id: *object_id,
            },
        )),
        _ => None,
    }
}

fn transform_list_size_directive(
    ctx: &mut Context<'_>,
    field_id: federated::FieldId,
    federated::ListSizeDirective {
        assumed_size,
        slicing_arguments,
        sized_fields,
        require_one_slicing_argument,
    }: federated::ListSizeDirective,
) -> federated::Directive {
    ctx.uses_list_size_directive = true;

    let field = &ctx.out[field_id];
    let argument_base_id = field.arguments.0;
    let arguments = &ctx.out[field.arguments];
    let slicing_arguments = slicing_arguments
        .iter()
        .filter_map(|argument| {
            let (index, _) = arguments
                .iter()
                .enumerate()
                .find(|(_, value)| ctx.lookup_string_id(value.name) == *argument)?;

            Some(federated::InputValueDefinitionId::from(
                index + usize::from(argument_base_id),
            ))
        })
        .collect();

    let child_type_id = field.r#type.definition;
    let sized_fields = sized_fields
        .iter()
        .filter_map(|field| {
            let field_name = ctx.lookup_str(field)?;
            ctx.selection_map.get(&(child_type_id, field_name)).copied()
        })
        .collect();

    federated::Directive::ListSize(federated::ListSize {
        assumed_size,
        slicing_arguments,
        sized_fields,
        require_one_slicing_argument,
    })
}

fn transform_authorized_entity_directive(
    ctx: &mut Context<'_>,
    parent: EntityDefinitionId,
    directive: ir::AuthorizedDirective,
) -> federated::Directive {
    let authorized = ctx.subgraphs.walk(directive.source).authorized().unwrap();
    let metadata = authorized.metadata.as_ref().map(|metadata| ctx.insert_value(metadata));
    let fields = authorized
        .fields
        .as_ref()
        .map(|fields| attach_selection(fields, parent.into(), ctx));

    federated::Directive::Authorized(federated::AuthorizedDirective {
        fields,
        node: None,
        arguments: None,
        metadata,
    })
}

fn transform_join_type_directive(
    ctx: &mut Context<'_>,
    parent: Definition,
    ir::JoinTypeDirective {
        subgraph_id,
        key,
        is_interface_object,
    }: ir::JoinTypeDirective,
) -> federated::Directive {
    if let Some(key) = key {
        let key = ctx.subgraphs.walk(key);
        let fields = attach_selection(key.fields(), parent, ctx);
        federated::Directive::JoinType(federated::JoinTypeDirective {
            subgraph_id,
            key: if fields.is_empty() { None } else { Some(fields) },
            is_interface_object,
            resolvable: key.is_resolvable(),
        })
    } else {
        federated::Directive::JoinType(federated::JoinTypeDirective {
            subgraph_id,
            key: None,
            resolvable: true,
            is_interface_object: false,
        })
    }
}

fn transform_join_field_directive(
    ctx: &mut Context<'_>,
    field_id: federated::FieldId,
    ir::JoinFieldDirective {
        source_field,
        r#override,
        override_label,
        r#type,
    }: ir::JoinFieldDirective,
) -> federated::Directive {
    let field = ctx.subgraphs.walk(source_field);
    federated::Directive::JoinField(federated::JoinFieldDirective {
        subgraph_id: Some(federated::SubgraphId::from(
            field.parent_definition().subgraph_id().idx(),
        )),
        requires: field
            .directives()
            .requires()
            .map(|field_set| attach_selection(field_set, ctx.out[field_id].parent_entity_id.into(), ctx)),
        provides: field
            .directives()
            .provides()
            .map(|field_set| attach_selection(field_set, ctx.out[field_id].r#type.definition, ctx)),
        r#type: r#type.map(|ty| ctx.insert_field_type(ctx.subgraphs.walk(ty))),
        r#override,
        override_label,
    })
}

fn transform_authorized_field_directive(
    ctx: &mut Context<'_>,
    field_id: federated::FieldId,
    directive: ir::AuthorizedDirective,
) -> federated::Directive {
    let directive = ctx.subgraphs.walk(directive.source).authorized().unwrap();
    let fields = directive
        .fields
        .as_ref()
        .map(|field_set| attach_selection(field_set, ctx.out[field_id].parent_entity_id.into(), ctx));
    let node = directive
        .node
        .as_ref()
        .map(|field_set| attach_selection(field_set, ctx.out[field_id].r#type.definition, ctx));
    let metadata = directive.metadata.as_ref().map(|metadata| ctx.insert_value(metadata));

    let arguments = directive
        .arguments
        .as_ref()
        .map(|args| attach_argument_selection(args, field_id, ctx));

    federated::Directive::Authorized(federated::AuthorizedDirective {
        fields,
        node,
        arguments,
        metadata,
    })
}

pub(super) fn emit_list_size_directive_definition(ctx: &mut Context<'_>) {
    if !ctx.uses_list_size_directive {
        return;
    }

    // directive @listSize(
    //   assumedSize: Int,
    //   slicingArguments: [String!],
    //   sizedFields: [String!],
    //   requireOneSlicingArgument: Boolean = true
    // ) on FIELD_DEFINITION

    let string_definition = ctx.definitions[&ctx.lookup_str("String").expect("String to be defined")];
    let int_definition = ctx.definitions[&ctx.lookup_str("Int").expect("Int to be defined")];
    let boolean_definition = ctx.definitions[&ctx.lookup_str("Boolean").expect("Boolean to be defined")];

    let name = ctx.insert_str("listSize");
    let assumed_size_str = ctx.insert_str("assumedSize");

    let directive_definition_id = ctx.out.push_directive_definition(federated::DirectiveDefinitionRecord {
        namespace: None,
        name,
        locations: federated::DirectiveLocations::FIELD_DEFINITION,
        repeatable: false,
    });

    ctx.out.push_directive_definition_argument(
        directive_definition_id,
        federated::InputValueDefinition {
            name: assumed_size_str,
            r#type: federated::Type {
                wrapping: Wrapping::new(false),
                definition: int_definition,
            },
            directives: Vec::new(),
            description: None,
            default: None,
        },
    );

    let slicing_arguments_str = ctx.insert_str("slicingArguments");

    ctx.out.push_directive_definition_argument(
        directive_definition_id,
        federated::InputValueDefinition {
            name: slicing_arguments_str,
            r#type: federated::Type {
                wrapping: Wrapping::required().wrap_list(),
                definition: string_definition,
            },
            directives: Vec::new(),
            description: None,
            default: None,
        },
    );

    let sized_fields_str = ctx.insert_str("sizedFields");

    ctx.out.push_directive_definition_argument(
        directive_definition_id,
        federated::InputValueDefinition {
            name: sized_fields_str,
            r#type: federated::Type {
                wrapping: Wrapping::required().wrap_list(),
                definition: string_definition,
            },
            directives: Vec::new(),
            description: None,
            default: None,
        },
    );

    let require_one_slicing_argument_str = ctx.insert_str("requireOneSlicingArgument");

    ctx.out.push_directive_definition_argument(
        directive_definition_id,
        federated::InputValueDefinition {
            name: require_one_slicing_argument_str,
            r#type: federated::Type {
                wrapping: Wrapping::new(false),
                definition: boolean_definition,
            },
            directives: Vec::new(),
            description: None,
            default: Some(federated::Value::Boolean(true)),
        },
    );
}

pub(super) fn emit_cost_directive_definition(ctx: &mut Context<'_>) {
    if !ctx.uses_cost_directive {
        return;
    }

    // directive @cost(weight: Int!) on
    //     ARGUMENT_DEFINITION
    //   | ENUM
    //   | FIELD_DEFINITION
    //   | INPUT_FIELD_DEFINITION
    //   | OBJECT
    //   | SCALAR

    let int_definition = ctx.definitions[&ctx.lookup_str("Int").expect("Int to be defined")];
    let name = ctx.insert_str("cost");
    let weight_str = ctx.insert_str("weight");

    let directive_definition_id = ctx.out.push_directive_definition(federated::DirectiveDefinitionRecord {
        namespace: None,
        name,
        locations: federated::DirectiveLocations::ARGUMENT_DEFINITION
            | federated::DirectiveLocations::ENUM
            | federated::DirectiveLocations::FIELD_DEFINITION
            | federated::DirectiveLocations::INPUT_FIELD_DEFINITION
            | federated::DirectiveLocations::OBJECT
            | federated::DirectiveLocations::SCALAR,
        repeatable: false,
    });

    ctx.out.push_directive_definition_argument(
        directive_definition_id,
        federated::InputValueDefinition {
            name: weight_str,
            r#type: federated::Type {
                wrapping: Wrapping::required(),
                definition: int_definition,
            },
            directives: Vec::new(),
            description: None,
            default: None,
        },
    );
}
