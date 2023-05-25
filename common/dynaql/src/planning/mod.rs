//! This module takes a parsed GraphQL query and uses the query_planning crate
//! to build a `LogicalQuery` - an intermediate representation of the query, that
//! contains `LogicalPlan`s.
//!
//! We can then go on to execute this `LogicalQuery` without needing access to
//! the original query or the `Registry` or any other dynaql specific bits.

use dynaql_parser::Positioned;
use query_planning::{
    logical_plan::{builder::LogicalPlanBuilder, LogicalPlan},
    logical_query::{
        ConditionSelectionSet, FieldPlan, SelectionPlan, SelectionPlanSet, TypeCondition,
    },
    reexport::{
        arrow_schema::{DataType, Field},
        internment::ArcIntern,
    },
    scalar::{graphql::GraphQLScalars, ScalarValue},
};

use crate::model::__Schema;
use crate::parser::types::Selection;

use crate::registry::utils::type_to_base_type;
use crate::registry::MetaType;
use crate::{ContextField, ContextSelectionSet, OutputType, ServerError, ServerResult};

use auth::AuthContext;

pub mod auth;

/// Builds a selection plan for the given query that executes all the top
/// level fields in parallel
pub async fn build_parallel_selection_plan<'a>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a MetaType,
    parent_logical_plan: Option<ArcIntern<LogicalPlan>>,
) -> ServerResult<Positioned<SelectionPlanSet>> {
    build_selection_plan_inner(ctx, true, root, parent_logical_plan).await
}

/// Builds a selection plan for the given query that executes all the top
/// level fields serially
pub async fn build_serial_selection_plan<'a>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a MetaType,
    parent_logical_plan: Option<ArcIntern<LogicalPlan>>,
) -> ServerResult<Positioned<SelectionPlanSet>> {
    build_selection_plan_inner(ctx, false, root, parent_logical_plan).await
}

async fn build_selection_plan_inner<'a>(
    ctx: &ContextSelectionSet<'a>,
    // TODO: Do we need this parameter?  It's unused but is that a mistake or
    // intentional?  I certainly can't tell
    _parallel: bool,
    root: &'a MetaType,
    parent_logical_plan: Option<ArcIntern<LogicalPlan>>,
) -> ServerResult<Positioned<SelectionPlanSet>> {
    Ok(ctx
        .item
        .position_node(plan_for_selection_set(ctx, root, parent_logical_plan).await?))
}

/// Takes a dynaql selection set and uses that to build a SelectionPlan for that
/// SelectionSet and all of it's children.
#[async_recursion::async_recursion]
async fn plan_for_selection_set(
    ctx: &ContextSelectionSet<'_>,
    root: &MetaType,
    parent_logical_plan: Option<ArcIntern<LogicalPlan>>,
) -> ServerResult<SelectionPlanSet> {
    #[cfg(feature = "tracing_worker")]
    {
        logworker::info!("", "Actual root {}", root.name());
    }
    let registry = ctx.registry();

    let mut result = vec![];

    for selection in &ctx.item.node.items {
        match &selection.node {
            Selection::Field(field) => {
                #[cfg(feature = "tracing_worker")]
                {
                    logworker::info!("", "field selected {}", field.node.name.node);
                }
                if field.node.name.node == "__typename" {
                    match root {
                        // When it's an interface, it means the typename will be infered based
                        // on the actual entity handled.
                        MetaType::Interface { .. } => {
                            let ctx_field = ctx.with_field(field, Some(root), Some(&ctx.item.node));

                            let plan = ctx.item.position_node(SelectionPlan::Field(
                                ctx.item.position_node(FieldPlan {
                                    nullable: false,
                                    array: false,
                                    ty: Some(GraphQLScalars::String),
                                    name: field.node.response_key().clone().map(|x| x.to_string()),
                                    logic_plan: LogicalPlanBuilder::from(
                                        ctx_field
                                            .to_logic_plan(root, parent_logical_plan.clone())?,
                                    )
                                    .projection(vec!["__type"])
                                    .expect("can't fail?")
                                    .build(),
                                    selection_set: Default::default(),
                                }),
                            ));

                            result.push(plan);
                            continue;
                        }
                        _ => {
                            // Get the typename
                            // The actual typename should be the concrete typename.
                            let ctx_field = ctx.with_field(field, Some(root), Some(&ctx.item.node));
                            let _field_name = ctx_field.item.node.name.node.clone();
                            let _alias = ctx_field.item.node.alias.clone().map(|x| x.node);
                            let typename = registry.introspection_type_name(root).to_owned();

                            let plan = ctx.item.position_node(SelectionPlan::Field(
                                ctx.item.position_node(FieldPlan {
                                    nullable: false,
                                    array: false,
                                    ty: Some(GraphQLScalars::String),
                                    name: field.node.response_key().clone().map(|x| x.to_string()),
                                    logic_plan: LogicalPlanBuilder::from(
                                        parent_logical_plan.clone().unwrap_or_else(|| {
                                            LogicalPlanBuilder::values(
                                                vec![Field::new("__type", DataType::Utf8, false)],
                                                vec![vec![ScalarValue::new_utf8(typename.clone())]],
                                            )
                                            .expect("can't fail")
                                            .build()
                                        }),
                                    )
                                    .projection_default(vec![(
                                        "__type",
                                        Some(ScalarValue::new_utf8(typename)),
                                        true,
                                    )])
                                    .expect("can't fail")
                                    .build(),
                                    selection_set: Default::default(),
                                }),
                            ));

                            result.push(plan);
                            continue;
                        }
                    }
                }

                let ctx_field = ctx.with_field(field, Some(root), Some(&ctx.item.node));

                let auth_ctx = AuthContext::new(&ctx_field);
                auth_ctx.check_resolving_logical_query(&ctx_field, root)?;

                let plan =
                    build_plan_for_field(&ctx_field, root, parent_logical_plan.clone()).await?;

                result.push(plan);
            }
            selection => {
                let (type_condition, selection_set) = match selection {
                    Selection::Field(_) => unreachable!(),
                    Selection::FragmentSpread(spread) => {
                        let fragment = ctx.query_env.fragments.get(&spread.node.fragment_name.node);
                        let fragment = match fragment {
                            Some(fragment) => fragment,
                            None => {
                                return Err(ServerError::new(
                                    format!(
                                        r#"Unknown fragment "{}"."#,
                                        spread.node.fragment_name.node
                                    ),
                                    Some(spread.pos),
                                ));
                            }
                        };
                        (
                            Some(&fragment.node.type_condition),
                            &fragment.node.selection_set,
                        )
                    }
                    Selection::InlineFragment(fragment) => (
                        fragment.node.type_condition.as_ref(),
                        &fragment.node.selection_set,
                    ),
                };
                let type_condition =
                    type_condition.map(|condition| condition.node.on.node.as_str());

                #[cfg(feature = "tracing_worker")]
                {
                    logworker::info!("", "on {}?", type_condition.unwrap_or_default());
                }

                let plan = match type_condition {
                    Some(type_condition) => {
                        vec![ctx.item.position_node(SelectionPlan::Condition(
                            ctx.item.position_node(ConditionSelectionSet {
                                type_condition: TypeCondition {
                                    on: ctx.item.position_node(type_condition.to_string()),
                                },
                                logic_plan: LogicalPlanBuilder::from(
                                    parent_logical_plan
                                        .clone()
                                        .ok_or_else(|| {
                                            ServerError::new(
                                                "todo: write error message",
                                                Some(ctx.item.pos),
                                            )
                                        })?
                                        .as_ref()
                                        .clone(),
                                )
                                .projection(vec!["__type"])
                                .expect("shouldn't fail")
                                .build(),
                                selection_set: {
                                    if selection_set.node.items.is_empty() {
                                        Default::default()
                                    } else {
                                        let associated_meta_ty =
                                            ctx.registry().types.get(type_condition).unwrap();
                                        let ctx_selection_set =
                                            ctx.with_selection_set(selection_set);

                                        build_parallel_selection_plan(
                                            &ctx_selection_set,
                                            associated_meta_ty,
                                            parent_logical_plan.clone(),
                                        )
                                        .await?
                                    }
                                },
                            }),
                        ))]
                    }
                    None => {
                        let ctx = ctx.with_selection_set(selection_set);
                        let plan =
                            plan_for_selection_set(&ctx, root, parent_logical_plan.clone()).await?;
                        ctx.item.position_node(plan).node.items
                    }
                };

                result.extend(plan);
            }
        }
    }
    Ok(SelectionPlanSet { items: result })
}

/// Function used to resolve a Field of a [`MetaType`] and return a Plan.
pub async fn build_plan_for_field<'a>(
    ctx: &'a ContextField<'a>,
    root: &'a MetaType,
    previous_logical_plan: Option<ArcIntern<LogicalPlan>>,
) -> ServerResult<Positioned<SelectionPlan>> {
    use query_planning::logical_query::dynaql::to_selection_plan;

    let field = ctx.item;

    // Need to be async for this old interop
    // TODO: When removing this, you have to remove the async pattern as it won't be usefull
    // anymore.
    if ctx.item.node.name.node == "__schema" {
        let ctx_obj = ctx.with_selection_set(&ctx.item.node.selection_set);
        let visible_types = ctx.schema_env.registry.find_visible_types(ctx);
        let node_id = OutputType::resolve(
            &__Schema::new(&ctx.schema_env.registry, &visible_types),
            &ctx_obj,
            ctx.item,
        )
        .await?;

        let mut graph = ctx.response_graph.write().await;

        let plan = to_selection_plan(
            field.node.response_key().node.as_str(),
            graph
                .take_node_into_const_value(node_id)
                .ok_or_else(|| {
                    ServerError::new(
                        "Internal error in introspection query",
                        Some(field.node.name.pos),
                    )
                })?
                .into(),
            ctx_obj.item.pos,
        );
        return Ok(plan);
    }

    let actual_logic_plan = ctx.to_logic_plan(root, previous_logical_plan)?;
    let associated_meta_field = root
        .field_by_name(field.node.name.node.as_str())
        .ok_or_else(|| {
            ServerError::new(
                format!("Can't find the associated field: {}", field.node.name),
                Some(field.node.name.pos),
            )
        })?;

    let selection_set = if !field.node.selection_set.node.items.is_empty() {
        let associated_meta_ty = ctx
            .registry()
            .types
            .get(&type_to_base_type(&associated_meta_field.ty).unwrap_or_default())
            .ok_or_else(|| {
                ServerError::new(
                    format!(
                        "Can't find the associated type: {}",
                        &associated_meta_field.ty
                    ),
                    Some(field.node.name.pos),
                )
            })?;
        let ctx_selection_set = ctx.with_selection_set(&field.node.selection_set);

        build_parallel_selection_plan(
            &ctx_selection_set,
            associated_meta_ty,
            Some(actual_logic_plan.clone()),
        )
        .await?
    } else {
        ctx.item.position_node(SelectionPlanSet::default())
    };

    use dynaql_parser::types::Type;
    use query_planning::scalar::graphql::as_graphql_scalar;
    let ty = Type::new(&associated_meta_field.ty).ok_or_else(|| {
        ServerError::new(
            format!(
                "Can't find the associated type for field: {}",
                field.node.name
            ),
            Some(field.node.name.pos),
        )
    })?;

    let plan = field.position_node(SelectionPlan::Field(ctx.item.position_node(FieldPlan {
        nullable: ty.nullable,
        ty: as_graphql_scalar(ty.base.to_base_type_str()),
        array: ty.base.is_list(),
        name: field.node.response_key().clone().map(|x| x.to_string()),
        logic_plan: actual_logic_plan,
        selection_set,
    })));

    Ok(plan)
}
