use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
};

use itertools::Itertools;
use schema::{DataSourceId, FieldId, FieldResolver, ResolverId};

use super::{plans::ExecutionPlansBuilder, ExecutionPlan, ExecutionPlans, PlanId};
use crate::{
    formatter::{ContextAwareDebug, FormatterContext, FormatterContextHolder},
    request::{
        Operation, OperationFieldId, OperationPath, OperationPathSegment, OperationSelection, OperationSelectionSet,
        ResolvedTypeCondition, TypeCondition,
    },
    response::{GraphqlError, ReadSelection, ReadSelectionSet},
    Engine,
};

#[derive(thiserror::Error, Debug)]
pub enum PrepareError {
    #[error("internal error: {0}")]
    InternalError(String),
}

impl From<PrepareError> for GraphqlError {
    fn from(err: PrepareError) -> Self {
        GraphqlError {
            message: err.to_string(),
            locations: vec![],
            path: vec![],
        }
    }
}

pub type PrepareResult<T> = Result<T, PrepareError>;

// This is the part that should be cached for a GraphQL query.
pub struct OperationPlan {
    pub operation: Operation,
    pub execution_plans: ExecutionPlans,
    pub final_read_selection_set: ReadSelectionSet,
}

impl OperationPlan {
    #[allow(clippy::unnecessary_wraps)]
    pub fn prepare(engine: &Engine, operation: Operation) -> PrepareResult<OperationPlan> {
        // Creating the final read selection set immediately before any modifications (plan inputs
        // adding fields)
        let final_read_selection_set = create_final_read_selection_set(&operation, &operation.selection_set);
        let mut planner = Planner {
            engine,
            operation,
            plans: ExecutionPlans::builder(),
        };
        planner.plan_operation();
        let operation = planner.operation;
        let execution_plans = planner.plans.build();

        Ok(OperationPlan {
            operation,
            execution_plans,
            final_read_selection_set,
        })
    }
}

fn create_final_read_selection_set(operation: &Operation, selection_set: &OperationSelectionSet) -> ReadSelectionSet {
    selection_set
        .iter()
        .map(|selection| {
            let op_field = &operation[selection.operation_field_id];
            ReadSelection {
                response_position: op_field.position,
                response_name: op_field.name,
                subselection: create_final_read_selection_set(operation, &selection.subselection),
            }
        })
        .collect()
}

pub struct Planner<'a> {
    engine: &'a Engine,
    operation: Operation,
    plans: ExecutionPlansBuilder,
}

impl<'a> Planner<'a> {
    fn plan_operation(&mut self) {
        let mut stack = vec![ToBePlanned {
            parent: None,
            path: OperationPath::empty(),
            selection_set: self.operation.selection_set.clone(),
        }];
        while let Some(to_be_planned) = stack.pop() {
            stack.extend(self.plan_next(to_be_planned));
        }
    }

    fn plan_next(
        &mut self,
        ToBePlanned {
            parent,
            path,
            mut selection_set,
        }: ToBePlanned,
    ) -> Vec<ToBePlanned> {
        if selection_set.is_empty() {
            return vec![];
        }

        #[derive(Debug, Default)]
        pub struct ResolverIO {
            input: schema::FieldSet,
            output: Vec<FieldId>,
        }

        let mut candidates = HashMap::<ResolverId, ResolverIO>::new();
        for selection in &selection_set {
            let field_id = self.operation[selection.operation_field_id].field_id;
            for FieldResolver { resolver_id, requires } in &self.engine.schema[field_id].resolvers {
                let io = candidates.entry(*resolver_id).or_default();
                io.input = schema::FieldSet::merge(&io.input, requires);
                io.output.push(field_id);
            }
        }

        // We assume no inputs and separate outputs for now.
        // Later we could:
        // - take candidate with least dependencies (to other plans) and provides the most fields
        // (probably?)
        // - plan it, and iterate until we planned everything.
        let mut plan_ids = vec![];
        let mut children = vec![];
        for (resolver_id, io) in candidates {
            assert!(io.input.is_empty());
            let resolver = &self.engine.schema[resolver_id];
            let (plan_selection_set, mut to_be_planned, rest) =
                self.partition_selection_set(resolver.data_source_id(), io.output, &path, selection_set);
            selection_set = rest;

            let plan_id = self.plans.push(ExecutionPlan {
                path: path.clone(),
                input: ReadSelectionSet::empty(),
                selection_set: plan_selection_set,
                resolver_id,
            });

            for plan in &mut to_be_planned {
                plan.parent = Some(plan_id);
            }
            children.extend(to_be_planned);
            plan_ids.push(plan_id);
        }

        if let Some(parent) = parent {
            for child in &plan_ids {
                self.plans.add_dependency(*child, parent);
            }
        }

        assert!(children.is_empty(), "CHILDREN:\n{:#?}", self.debug(&children));
        children
    }

    // will be used in a later PR.
    #[allow(dead_code)]
    fn compute_dense_capacity(&self, fields: impl IntoIterator<Item = OperationFieldId>) -> Option<NonZeroUsize> {
        let mut count: usize = 0;
        let mut type_conditions = HashSet::new();
        for id in fields {
            count += 1;
            type_conditions.insert(self.operation[id].type_condition);
        }
        // If we have more than one type condition, it means certain fields are optional. None
        // counts as 1. When using dense objects we don't filter fields so it must represent the
        // final state.
        if type_conditions.len() > 1 {
            None
        } else {
            NonZeroUsize::new(count)
        }
    }

    fn partition_selection_set(
        &self,
        data_source_id: DataSourceId,
        mut output_fields: Vec<FieldId>,
        path: &OperationPath,
        selection_set: OperationSelectionSet,
    ) -> (OperationSelectionSet, Vec<ToBePlanned>, OperationSelectionSet) {
        output_fields.sort_unstable();
        let mut to_be_planned = vec![];

        let (selection_set, rest) = selection_set
            .into_iter()
            .map(|selection| {
                let op_field = &self.operation[selection.operation_field_id];
                if output_fields.binary_search(&op_field.field_id).is_ok() {
                    let schema_field = &self.engine.schema[op_field.field_id];

                    Ok(OperationSelection {
                        operation_field_id: selection.operation_field_id,
                        subselection: self.assign_provideable_write_selection_set(
                            data_source_id,
                            schema_field.provides(data_source_id).cloned().unwrap_or_default(),
                            true,
                            self.make_path_child(path, selection.operation_field_id),
                            selection.subselection,
                            &mut to_be_planned,
                        ),
                    })
                } else {
                    Err(selection)
                }
            })
            .partition_result();

        (selection_set, to_be_planned, rest)
    }

    fn assign_provideable_write_selection_set(
        &self,
        data_source_id: DataSourceId,
        provideable: schema::FieldSet,
        assign_without_resolvers: bool,
        path: OperationPath,
        selection_set: OperationSelectionSet,
        to_be_planned: &mut Vec<ToBePlanned>,
    ) -> OperationSelectionSet {
        let (found, missing): (OperationSelectionSet, OperationSelectionSet) = selection_set
            .into_iter()
            .map(|selection| {
                let op_field = &self.operation[selection.operation_field_id];
                let schema_field = &self.engine.schema[op_field.field_id];

                let provideable_field = provideable.selection(op_field.field_id);
                if provideable_field.is_some() || (assign_without_resolvers && schema_field.resolvers.is_empty()) {
                    let provideable = schema::FieldSet::merge_opt(
                        provideable_field.map(|s| &s.subselection),
                        schema_field.provides(data_source_id),
                    );
                    Ok(OperationSelection {
                        operation_field_id: selection.operation_field_id,
                        subselection: self.assign_provideable_write_selection_set(
                            data_source_id,
                            provideable,
                            schema_field.resolvers.is_empty(),
                            self.make_path_child(&path, selection.operation_field_id),
                            selection.subselection,
                            to_be_planned,
                        ),
                    })
                } else {
                    Err(selection)
                }
            })
            .partition_result();
        if !missing.is_empty() {
            to_be_planned.push(ToBePlanned {
                parent: None, // defined later
                path,
                selection_set: missing,
            });
        }
        found
    }

    fn make_path_child(&self, parent: &OperationPath, child: OperationFieldId) -> OperationPath {
        let reponse_field = &self.operation[child];
        parent.child(OperationPathSegment {
            operation_field_id: child,
            type_condition: reponse_field.type_condition.map(|cond| {
                ResolvedTypeCondition::new(match cond {
                    TypeCondition::Interface(interface_id) => self.engine.schema[interface_id].possible_types.clone(),
                    TypeCondition::Object(object_id) => vec![object_id],
                    TypeCondition::Union(union_id) => self.engine.schema[union_id].possible_types.clone(),
                })
            }),
            position: reponse_field.position,
            name: reponse_field.name,
        })
    }
}

#[derive(Debug)]
struct ToBePlanned {
    parent: Option<PlanId>,
    path: OperationPath,
    selection_set: OperationSelectionSet,
}

impl<'a> FormatterContextHolder for Planner<'a> {
    fn formatter_context(&self) -> FormatterContext<'_> {
        FormatterContext {
            schema: &self.engine.schema,
            strings: &self.operation.strings,
            opeartion: &self.operation,
        }
    }
}

impl ContextAwareDebug for ToBePlanned {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToBePlanned")
            .field("parent", &self.parent)
            .field("path", &ctx.debug(&self.path))
            .field("selection_set", &ctx.debug(&self.selection_set))
            .finish()
    }
}
