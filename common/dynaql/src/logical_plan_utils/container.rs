use dynaql_parser::Positioned;
use futures_util::FutureExt;

use query_planning::logical_plan::builder::LogicalPlanBuilder;
use query_planning::logical_plan::LogicalPlan;
use query_planning::logical_query::{
    ConditionSelectionSet, FieldPlan, SelectionPlan, SelectionPlanSet, TypeCondition,
};
use query_planning::reexport::arrow_schema::{DataType, Field};
use query_planning::scalar::ScalarValue;

use std::sync::Arc;

use crate::parser::types::Selection;
use crate::registry::utils::type_to_base_type;
use crate::registry::MetaType;
use crate::{ContextSelectionSet, OutputType, ServerError, ServerResult};

/// Resolve an container by executing each of the fields concurrently.
pub fn resolve_logical_plan_container<'a>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a MetaType,
    previous_logical_plan: Option<Arc<LogicalPlan>>,
) -> ServerResult<Positioned<SelectionPlanSet>> {
    resolve_logical_plan_container_inner(ctx, true, root, previous_logical_plan)
}

/// Resolve an container by executing each of the fields serially.
pub fn resolve_logical_plan_container_serial<'a>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a MetaType,
    previous_logical_plan: Option<Arc<LogicalPlan>>,
) -> ServerResult<Positioned<SelectionPlanSet>> {
    resolve_logical_plan_container_inner(ctx, false, root, previous_logical_plan)
}

fn resolve_logical_plan_container_inner<'a>(
    ctx: &ContextSelectionSet<'a>,
    _parallel: bool,
    root: &'a MetaType,
    previous_logical_plan: Option<Arc<LogicalPlan>>,
) -> ServerResult<Positioned<SelectionPlanSet>> {
    Ok(ctx
        .item
        .position_node(FieldsGraph::add_set(ctx, root, previous_logical_plan)?))
}

type BoxFieldGraphFuture = ServerResult<SelectionPlan>;
pub struct FieldsGraph(Vec<BoxFieldGraphFuture>);

impl FieldsGraph {
    /// Add another set of fields to this set of fields using the given container.
    pub fn add_set(
        ctx: &ContextSelectionSet<'_>,
        root: &MetaType,
        previous_logical_plan: Option<Arc<LogicalPlan>>,
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
                                let ctx_field =
                                    ctx.with_field(field, Some(root), Some(&ctx.item.node));

                                let plan = ctx.item.position_node(SelectionPlan::Field(
                                    ctx.item.position_node(FieldPlan {
                                        name: field
                                            .node
                                            .response_key()
                                            .clone()
                                            .map(|x| x.to_string()),
                                        logic_plan: LogicalPlanBuilder::from(
                                            ctx_field.to_logic_plan(
                                                root,
                                                previous_logical_plan.clone(),
                                            )?,
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
                                let ctx_field =
                                    ctx.with_field(field, Some(root), Some(&ctx.item.node));
                                let _field_name = ctx_field.item.node.name.node.clone();
                                let _alias = ctx_field.item.node.alias.clone().map(|x| x.node);
                                let typename = registry.introspection_type_name(root).to_owned();

                                let plan = ctx.item.position_node(SelectionPlan::Field(
                                    ctx.item.position_node(FieldPlan {
                                        name: field
                                            .node
                                            .response_key()
                                            .clone()
                                            .map(|x| x.to_string()),
                                        logic_plan: LogicalPlanBuilder::values(
                                            vec![Field::new("__type", DataType::Utf8, false)],
                                            vec![vec![ScalarValue::new_utf8(typename)]],
                                        )
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
                    let actual_logic_plan =
                        ctx_field.to_logic_plan(root, previous_logical_plan.clone())?;
                    let selection_set = if !field.node.selection_set.node.items.is_empty() {
                        let associated_meta_field = root
                            .field_by_name(field.node.name.node.as_str())
                            .ok_or_else(|| {
                                ServerError::new(
                                    format!("Can't find the associated field: {}", field.node.name),
                                    Some(field.node.name.pos),
                                )
                            })?;
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
                        let ctx_selection_set =
                            ctx_field.with_selection_set(&field.node.selection_set);
                        

                        resolve_logical_plan_container(
                            &ctx_selection_set,
                            associated_meta_ty,
                            Some(Arc::new(actual_logic_plan.clone())),
                        )?
                    } else {
                        ctx_field.item.position_node(SelectionPlanSet::default())
                    };

                    let plan = ctx_field.item.position_node(SelectionPlan::Field(
                        ctx_field.item.position_node(FieldPlan {
                            name: field.node.response_key().clone().map(|x| x.to_string()),
                            logic_plan: actual_logic_plan,
                            selection_set,
                        }),
                    ));
                    result.push(plan);
                }
                selection => {
                    let (type_condition, selection_set) = match selection {
                        Selection::Field(_) => unreachable!(),
                        Selection::FragmentSpread(spread) => {
                            let fragment =
                                ctx.query_env.fragments.get(&spread.node.fragment_name.node);
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
                                        previous_logical_plan
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
                                        if !selection_set.node.items.is_empty() {
                                            let associated_meta_ty =
                                                ctx.registry().types.get(type_condition).unwrap();
                                            let ctx_selection_set =
                                                ctx.with_selection_set(selection_set);
                                            
                                            resolve_logical_plan_container(
                                                &ctx_selection_set,
                                                associated_meta_ty,
                                                previous_logical_plan.clone(),
                                            )?
                                        } else {
                                            Default::default()
                                        }
                                    },
                                }),
                            ))]
                        }
                        None => {
                            let ctx = ctx.with_selection_set(selection_set);
                            let plan =
                                FieldsGraph::add_set(&ctx, root, previous_logical_plan.clone())?;
                            ctx.item.position_node(plan).node.items
                        }
                    };

                    result.extend(plan);
                }
            }
        }
        Ok(SelectionPlanSet { items: result })
    }
}
