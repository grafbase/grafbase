use wrapping::Wrapping;

use crate::composition_ir as ir;
use crate::federated_graph::{self as federated, Definition};

use super::{
    attach_selection,
    context::{Context, UsedDirectives},
};

pub(super) fn transform_arbitray_type_directives(
    ctx: &mut Context<'_>,
    directives: &[ir::Directive],
) -> Vec<federated::Directive> {
    directives
        .iter()
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
    directives: &[ir::Directive],
) -> Vec<federated::Directive> {
    directives
        .iter()
        .filter_map(|directive| match directive {
            ir::Directive::JoinInputField(dir) => {
                Some(federated::Directive::JoinField(federated::JoinFieldDirective {
                    subgraph_id: Some(dir.subgraph_id),
                    requires: None,
                    provides: None,
                    r#type: dir.r#type.map(|ty| ctx.insert_field_type(ctx.subgraphs.walk(ty))),
                    external: false,
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
    directives: &[ir::Directive],
) -> Vec<federated::Directive> {
    directives
        .iter()
        .filter_map(|directive| transform_common_directive(ctx, directive))
        .collect()
}

pub(super) fn transform_type_directives(
    ctx: &mut Context<'_>,
    parent: federated::Definition,
    directives: &[ir::Directive],
) -> Vec<federated::Directive> {
    directives
        .iter()
        .filter_map(|directive| match (directive, parent) {
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
    directives: &[ir::Directive],
) -> Vec<federated::Directive> {
    directives
        .iter()
        .filter_map(|directive| match directive {
            ir::Directive::JoinField(dir) => Some(transform_join_field_directive(ctx, field_id, dir)),
            ir::Directive::JoinEntityInterfaceField => {
                Some(federated::Directive::JoinField(federated::JoinFieldDirective::default()))
            }
            ir::Directive::ListSize(dir) => Some(transform_list_size_directive(ctx, field_id, dir)),
            dir => transform_common_directive(ctx, dir),
        })
        .collect()
}

fn transform_common_directive(ctx: &mut Context<'_>, directive: &ir::Directive) -> Option<federated::Directive> {
    Some(match directive {
        ir::Directive::Authenticated => federated::Directive::Authenticated,
        ir::Directive::Deprecated { reason } => federated::Directive::Deprecated { reason: *reason },
        ir::Directive::Inaccessible => federated::Directive::Inaccessible,
        ir::Directive::Policy(policies) => federated::Directive::Policy(policies.clone()),
        ir::Directive::RequiresScopes(scopes) => federated::Directive::RequiresScopes(scopes.clone()),

        ir::Directive::Cost { weight } => {
            ctx.used_directives |= UsedDirectives::COST;
            federated::Directive::Cost { weight: *weight }
        }

        ir::Directive::CompositeLookup(subgraph_id) => {
            ctx.used_directives |= UsedDirectives::COMPOSITE_LOOKUP;
            federated::Directive::CompositeLookup { graph: *subgraph_id }
        }
        ir::Directive::CompositeDerive(subgraph_id) => {
            ctx.used_directives |= UsedDirectives::COMPOSITE_DERIVE;
            federated::Directive::CompositeDerive { graph: *subgraph_id }
        }
        ir::Directive::CompositeInternal(subgraph_id) => {
            ctx.used_directives |= UsedDirectives::COMPOSITE_INTERNAL;
            federated::Directive::CompositeInternal { graph: *subgraph_id }
        }
        ir::Directive::CompositeRequire { subgraph_id, field } => {
            ctx.used_directives |= UsedDirectives::COMPOSITE_REQUIRE;
            let field = ctx.insert_string(ctx.subgraphs.walk(*field));
            federated::Directive::CompositeRequire {
                graph: *subgraph_id,
                field,
            }
        }
        ir::Directive::CompositeIs { subgraph_id, field } => {
            ctx.used_directives |= UsedDirectives::COMPOSITE_IS;
            let field = ctx.insert_string(ctx.subgraphs.walk(*field));
            federated::Directive::CompositeIs {
                graph: *subgraph_id,
                field,
            }
        }
        ir::Directive::Other {
            name,
            arguments,
            provenance: ir::DirectiveProvenance::Builtin | ir::DirectiveProvenance::ComposeDirective,
        } => federated::Directive::Other {
            name: *name,
            arguments: arguments
                .iter()
                .map(|(name, value)| (*name, ctx.insert_value(value)))
                .collect(),
        },
        ir::Directive::OneOf => federated::Directive::OneOf,
        ir::Directive::Other {
            name,
            arguments,
            provenance:
                ir::DirectiveProvenance::LinkedFromExtension {
                    linked_schema_id,
                    extension_id,
                },
        } => {
            let subgraph_id = ctx.subgraphs.at(*linked_schema_id).subgraph_id;
            let arguments = arguments
                .iter()
                .map(|(name, value)| (*name, ctx.insert_value(value)))
                .collect();
            let extension_id = ctx.convert_extension_id(*extension_id);

            federated::Directive::ExtensionDirective(federated::ExtensionDirective {
                subgraph_id: federated::SubgraphId::from(subgraph_id.idx()),
                extension_id,
                name: *name,
                arguments: Some(arguments),
            })
        }
        ir::Directive::JoinField(_)
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
    ir::JoinUnionMemberDirective { member }: &ir::JoinUnionMemberDirective,
) -> Option<federated::Directive> {
    let member = ctx.subgraphs.walk(*member);
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
    }: &federated::ListSizeDirective,
) -> federated::Directive {
    ctx.used_directives |= UsedDirectives::LIST_SIZE;

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
        assumed_size: *assumed_size,
        slicing_arguments,
        sized_fields,
        require_one_slicing_argument: *require_one_slicing_argument,
    })
}

fn transform_join_type_directive(
    ctx: &mut Context<'_>,
    parent: Definition,
    ir::JoinTypeDirective {
        subgraph_id,
        key,
        is_interface_object,
    }: &ir::JoinTypeDirective,
) -> federated::Directive {
    if let Some(key) = key {
        let key = ctx.subgraphs.walk(*key);
        let fields = attach_selection(key.fields(), parent, ctx);
        federated::Directive::JoinType(federated::JoinTypeDirective {
            subgraph_id: *subgraph_id,
            key: if fields.is_empty() { None } else { Some(fields) },
            is_interface_object: *is_interface_object,
            resolvable: key.is_resolvable(),
        })
    } else {
        federated::Directive::JoinType(federated::JoinTypeDirective {
            subgraph_id: *subgraph_id,
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
        external,
        r#override,
        override_label,
        r#type,
    }: &ir::JoinFieldDirective,
) -> federated::Directive {
    let field = ctx.subgraphs.walk(*source_field);
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
        external: *external,
        r#override: r#override.clone(),
        override_label: override_label.clone(),
    })
}

pub(super) fn emit_list_size_directive_definition(ctx: &mut Context<'_>) {
    if !ctx.used_directives.contains(UsedDirectives::LIST_SIZE) {
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
                wrapping: Wrapping::default(),
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
                wrapping: Wrapping::default().non_null().list(),
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
                wrapping: Wrapping::default().non_null().list(),
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
                wrapping: Wrapping::default(),
                definition: boolean_definition,
            },
            directives: Vec::new(),
            description: None,
            default: Some(federated::Value::Boolean(true)),
        },
    );
}

pub(super) fn emit_cost_directive_definition(ctx: &mut Context<'_>) {
    if !ctx.used_directives.contains(UsedDirectives::COST) {
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
                wrapping: Wrapping::default().non_null(),
                definition: int_definition,
            },
            directives: Vec::new(),
            description: None,
            default: None,
        },
    );
}

pub(super) fn emit_composite_spec_directive_definitions(ctx: &mut Context<'_>) {
    let composite_namespace = Some(ctx.insert_str("composite"));
    let lookup_str = ctx.insert_str("lookup");
    let require_str = ctx.insert_str("require");
    let is_str = ctx.insert_str("is");
    let field_str = ctx.insert_str("field");
    let internal_str = ctx.insert_str("internal");

    if ctx.used_directives.contains(UsedDirectives::COMPOSITE_LOOKUP) {
        // directive @lookup on FIELD_DEFINITION
        //
        // see https://github.com/graphql/composite-schemas-spec/blob/main/spec/Appendix%20A%20--%20Field%20Selection.md#lookup

        ctx.out.push_directive_definition(federated::DirectiveDefinitionRecord {
            namespace: composite_namespace,
            name: lookup_str,
            locations: federated::DirectiveLocations::FIELD_DEFINITION,
            repeatable: false,
        });
    }

    let field_selection_map_id = if ctx.used_directives.contains(UsedDirectives::COMPOSITE_REQUIRE)
        || ctx.used_directives.contains(UsedDirectives::COMPOSITE_IS)
    {
        // composite__FieldSelectionMap
        let name = ctx.insert_str("FieldSelectionMap");
        Some(ctx.out.push_scalar_definition(federated::ScalarDefinitionRecord {
            namespace: composite_namespace,
            name,
            directives: Vec::new(),
            description: None,
        }))
    } else {
        None
    };

    if ctx.used_directives.contains(UsedDirectives::COMPOSITE_REQUIRE) {
        // directive @require(field: FieldSelectionMap!) on ARGUMENT_DEFINITION
        //
        // See https://github.com/graphql/composite-schemas-spec/blob/main/spec/Appendix%20A%20--%20Field%20Selection.md#require

        let directive_definition_id = ctx.out.push_directive_definition(federated::DirectiveDefinitionRecord {
            namespace: composite_namespace,
            name: require_str,
            locations: federated::DirectiveLocations::ARGUMENT_DEFINITION,
            repeatable: false,
        });

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: field_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default().non_null(),
                    definition: federated::Definition::Scalar(field_selection_map_id.unwrap()),
                },
                directives: vec![],
                description: None,
                default: None,
            },
        );
    }

    if ctx.used_directives.contains(UsedDirectives::COMPOSITE_IS) {
        // directive @is(field: FieldSelectionMap!) on ARGUMENT_DEFINITION | FIELD_DEFINITION
        //
        // field is specific to Grafbase for computed fields.
        // See https://github.com/graphql/composite-schemas-spec/blob/main/spec/Section%202%20--%20Source%20Schema.md#is

        let directive_definition_id = ctx.out.push_directive_definition(federated::DirectiveDefinitionRecord {
            namespace: composite_namespace,
            name: is_str,
            locations: federated::DirectiveLocations::ARGUMENT_DEFINITION
                | federated::DirectiveLocations::FIELD_DEFINITION,
            repeatable: false,
        });

        ctx.out.push_directive_definition_argument(
            directive_definition_id,
            federated::InputValueDefinition {
                name: field_str,
                r#type: federated::Type {
                    wrapping: Wrapping::default().non_null(),
                    definition: federated::Definition::Scalar(field_selection_map_id.unwrap()),
                },
                directives: vec![],
                description: None,
                default: None,
            },
        );
    }

    if ctx.used_directives.contains(UsedDirectives::COMPOSITE_INTERNAL) {
        // https://github.com/graphql/composite-schemas-spec/blob/main/spec/Section%202%20--%20Source%20Schema.md#internal
        // directive @internal on OBJECT | FIELD_DEFINITION

        ctx.out.push_directive_definition(federated::DirectiveDefinitionRecord {
            namespace: composite_namespace,
            name: internal_str,
            locations: federated::DirectiveLocations::OBJECT | federated::DirectiveLocations::FIELD_DEFINITION,
            repeatable: false,
        });
    }
}
