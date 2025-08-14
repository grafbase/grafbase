mod context;
mod directive_definitions;
mod directives;
mod entity_interface;
mod enums;
mod fields;
mod input_object;
mod interface;
mod object;
mod reserved_names;
mod roots;
mod scalar;

pub(crate) use self::context::Context as ComposeContext;

use self::{context::Context, directives::collect_composed_directives, input_object::*};
use crate::{
    composition_ir as ir,
    diagnostics::CompositeSchemasPreMergeValidationErrorCode,
    federated_graph as federated,
    subgraphs::{self, DefinitionKind, DefinitionWalker, FieldWalker, StringId},
};
use directives::create_join_type_from_definitions;
use itertools::Itertools;
use std::collections::{BTreeSet, HashSet};

pub(crate) fn compose_subgraphs(ctx: &mut Context<'_>) {
    ctx.subgraphs.iter_definition_groups(|definitions| {
        let Some(first) = definitions.first() else {
            return;
        };

        if reserved_names::validate_definition_names(definitions, ctx) {
            return;
        };

        if first.directives().interface_object()
            || (first.kind() == DefinitionKind::Interface && first.entity_keys().next().is_some())
        {
            return entity_interface::merge_entity_interface_definitions(ctx, *first, definitions);
        }

        match first.kind() {
            DefinitionKind::Object => merge_object_definitions(ctx, first, definitions),
            DefinitionKind::Union => merge_union_definitions(ctx, first, definitions),
            DefinitionKind::InputObject => merge_input_object_definitions(ctx, first, definitions),
            DefinitionKind::Interface => interface::merge_interface_definitions(ctx, first, definitions),
            DefinitionKind::Scalar => scalar::merge_scalar_definitions(*first, definitions, ctx),
            DefinitionKind::Enum => enums::merge_enum_definitions(first, definitions, ctx),
        }
    });

    roots::merge_root_fields(ctx);
    directive_definitions::compose_directive_definitions(ctx);
}

fn merge_object_definitions<'a>(
    ctx: &mut Context<'a>,
    first: &DefinitionWalker<'a>,
    definitions: &[DefinitionWalker<'a>],
) {
    let is_shareable = definitions.iter().any(|definition| definition.directives().shareable());

    if let Some(incompatible) = definitions
        .iter()
        .find(|definition| definition.kind() != DefinitionKind::Object)
    {
        let first_kind = first.kind();
        let second_kind = incompatible.kind();
        let name = first.name().as_str();
        let first_subgraph = first.subgraph().name().as_str();
        let second_subgraph = incompatible.subgraph().name().as_str();
        ctx.diagnostics.push_composite_schemas_pre_merge_validation_error(format!(
            "Cannot merge {first_kind:?} with {second_kind:?} (`{name}` in `{first_subgraph}` and `{second_subgraph}`)",
        ), CompositeSchemasPreMergeValidationErrorCode::TypeKindMismatch);
        return;
    }

    let is_entity = first.is_entity();
    if definitions.iter().any(|object| object.is_entity() != is_entity) {
        let name = first.name().as_str();
        let (entity_subgraphs, non_entity_subgraphs) = definitions
            .iter()
            .partition::<Vec<DefinitionWalker<'_>>, _>(|definition| definition.is_entity());

        ctx.diagnostics.push_fatal(format!(
            "The `{name}` object is an entity in subgraphs {} but not in subgraphs {}.",
            entity_subgraphs
                .into_iter()
                .map(|d| d.subgraph().name().as_str())
                .join(", "),
            non_entity_subgraphs
                .into_iter()
                .map(|d| d.subgraph().name().as_str())
                .join(", "),
        ));
    }

    let description = definitions.iter().find_map(|def| def.description());
    let mut directives = collect_composed_directives(definitions.iter().map(|def| def.directives()), ctx);

    if is_entity {
        directives.extend(definitions.iter().flat_map(|def| def.entity_keys()).map(|key| {
            ir::Directive::JoinType(ir::JoinTypeDirective {
                subgraph_id: federated::SubgraphId::from(key.parent_definition().subgraph_id().idx()),
                key: Some(key.id),
                is_interface_object: false,
            })
        }));
    } else {
        directives.extend(create_join_type_from_definitions(definitions));
    }
    let object_name = ctx.insert_string(first.name().id);
    ctx.insert_object(object_name, description, directives);

    if is_shareable {
        object::validate_shareable_object_fields_match(definitions, ctx);
    }

    let fields = object::compose_fields(ctx, definitions, object_name, is_shareable);
    for field in fields {
        ctx.insert_field(field);
    }
}

fn merge_union_definitions(
    ctx: &mut Context<'_>,
    first_union: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
) {
    let union_name = ctx.insert_string(first_union.name().id);

    let description = definitions.iter().find_map(|def| def.description());
    let mut directives = collect_composed_directives(definitions.iter().map(|def| def.directives()), ctx);

    for member in definitions
        .iter()
        .flat_map(|def| ctx.subgraphs.iter_union_members(def.id))
    {
        directives.push(ir::Directive::JoinUnionMember(ir::JoinUnionMemberDirective { member }));

        let member = first_union.walk(member);
        let member_name = ctx.insert_string(member.name().id);
        ctx.insert_union_member(union_name, member_name);
    }

    ctx.insert_union(union_name, directives, description);
}
