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
mod validate;

pub(crate) use self::context::Context as ComposeContext;

use self::{context::Context, directives::collect_composed_directives, input_object::*};
use crate::{
    composition_ir as ir,
    diagnostics::CompositeSchemasPreMergeValidationErrorCode,
    federated_graph as federated,
    subgraphs::{self, DefinitionKind, DefinitionView, StringId},
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

        if entity_interface::is_entity_interface(ctx.subgraphs, definitions.iter().map(|def| def.id)) {
            return entity_interface::merge_entity_interface_definitions(ctx, *first, definitions);
        }

        match first.kind {
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

fn merge_object_definitions<'a>(ctx: &mut Context<'a>, first: &DefinitionView<'a>, definitions: &[DefinitionView<'a>]) {
    let is_shareable = definitions
        .iter()
        .any(|definition| definition.directives.shareable(ctx.subgraphs));

    if let Some(incompatible) = definitions
        .iter()
        .find(|definition| definition.kind != DefinitionKind::Object)
    {
        let first_kind = first.kind;
        let second_kind = incompatible.kind;
        let name = ctx.subgraphs[first.name].as_ref();
        let first_subgraph = ctx.subgraphs[ctx.subgraphs.at(first.subgraph_id).name].as_ref();
        let second_subgraph = ctx.subgraphs[ctx.subgraphs.at(incompatible.subgraph_id).name].as_ref();
        ctx.diagnostics.push_composite_schemas_pre_merge_validation_error(format!(
            "Cannot merge {first_kind:?} with {second_kind:?} (`{name}` in `{first_subgraph}` and `{second_subgraph}`)",
        ), CompositeSchemasPreMergeValidationErrorCode::TypeKindMismatch);
        return;
    }

    let is_entity = validate_consistent_entityness(ctx, definitions);

    let description = definitions
        .iter()
        .find_map(|def| def.description)
        .map(|desc| ctx.subgraphs[desc].as_ref());
    let mut directives = collect_composed_directives(definitions.iter().map(|def| def.directives), ctx);

    if is_entity {
        directives.extend(
            definitions
                .iter()
                .flat_map(|def| def.id.keys(ctx.subgraphs))
                .map(|key| {
                    ir::Directive::JoinType(ir::JoinTypeDirective {
                        subgraph_id: federated::SubgraphId::from(ctx.subgraphs.at(key.definition_id).subgraph_id.idx()),
                        key: Some(key.id),
                        is_interface_object: false,
                    })
                }),
        );
    } else {
        directives.extend(create_join_type_from_definitions(definitions));
    }
    let object_name = ctx.insert_string(first.name);
    ctx.insert_object(object_name, description, directives);

    if is_shareable {
        object::validate_shareable_object_fields_match(definitions, ctx);
    }

    let fields = object::compose_fields(ctx, definitions, object_name);
    for field in fields {
        ctx.insert_field(field);
    }
}

fn validate_consistent_entityness(ctx: &mut Context<'_>, definitions: &[DefinitionView<'_>]) -> bool {
    let is_entity = definitions.iter().any(|def| def.id.is_entity(ctx.subgraphs));

    if !is_entity {
        return false;
    }

    if definitions
        .iter()
        .all(|def| def.id.is_entity(ctx.subgraphs) || ctx.subgraphs.at(def.subgraph_id).federation_spec.is_apollo_v1())
    {
        return true;
    }

    let Some(definition) = definitions.first() else {
        return false;
    };
    let mut non_entity_fed_v2 = Vec::new();
    let mut entity_definitions = Vec::new();

    for definition in definitions {
        let definition = ctx.subgraphs.at(definition.id);
        let subgraph = ctx.subgraphs.at(definition.subgraph_id);
        let definition_is_entity = definition.id.is_entity(ctx.subgraphs);
        let subgraph_name = &ctx.subgraphs[subgraph.name];

        if definition_is_entity {
            entity_definitions.push(subgraph_name);
        } else if subgraph.federation_spec.is_apollo_v2() {
            non_entity_fed_v2.push(subgraph_name);
        }
    }

    ctx.diagnostics.push_fatal(format!(
        "The `{name}` object is an entity in subgraphs {entity_definitions} but not in subgraphs {non_entity_subgraphs}.",
        name = ctx.subgraphs[definition.name],
        entity_definitions = entity_definitions
            .into_iter()
            .join(", "),
        non_entity_subgraphs = non_entity_fed_v2
            .into_iter()
            .join(", "),
    ));

    false
}

fn merge_union_definitions(
    ctx: &mut Context<'_>,
    first_union: &DefinitionView<'_>,
    definitions: &[DefinitionView<'_>],
) {
    let union_name = ctx.insert_string(first_union.name);

    let description = definitions
        .iter()
        .find_map(|def| def.description)
        .map(|desc| ctx.subgraphs[desc].as_ref());
    let mut directives = collect_composed_directives(definitions.iter().map(|def| def.directives), ctx);

    for member in definitions
        .iter()
        .flat_map(|def| ctx.subgraphs.iter_union_members(def.id))
    {
        directives.push(ir::Directive::JoinUnionMember(ir::JoinUnionMemberDirective { member }));

        let member_name = ctx.insert_string(ctx.subgraphs.at(member).name);
        ctx.insert_union_member(union_name, member_name);
    }

    ctx.insert_union(union_name, directives, description);
}
