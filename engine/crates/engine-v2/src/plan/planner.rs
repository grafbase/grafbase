use std::collections::HashMap;

use engine_parser::{types::OperationType, Pos};
use itertools::Itertools;
use schema::{DataSourceId, FieldId, FieldResolver, ResolverId};

use super::{plans::ExecutionPlansBuilder, ExecutionPlan, ExecutionPlans, PlanId};
use crate::{
    request::OperationDefinition,
    response::{
        ReadSelection, ReadSelectionSet, ResponseFields, ResponseFieldsBuilder, ResponsePath, SelectionSet,
        TypeCondition, WriteSelection, WriteSelectionSet,
    },
    Engine,
};

// This is the part that should be cached for a GraphQL query.
// I suppose we might want to plan all query operations together if we support operation batching.
// we would just need to keep track of the operation name to the selection_set.
// If that doesn't make any sense, we should rename that to OperationPlan
pub struct RequestPlan {
    pub operation_type: OperationType,
    pub operation_selection_set: SelectionSet,
    pub response_fields: ResponseFields,
    pub execution_plans: ExecutionPlans,
}

impl RequestPlan {
    pub fn builder(engine: &Engine) -> RequestPlanBuilder<'_> {
        RequestPlanBuilder {
            engine,
            plans: ExecutionPlans::builder(),
            response_fields: ResponseFields::builder(),
        }
    }
}

pub struct RequestPlanBuilder<'a> {
    engine: &'a Engine,
    response_fields: ResponseFieldsBuilder,
    plans: ExecutionPlansBuilder,
}

impl<'a> RequestPlanBuilder<'a> {
    pub fn build(mut self, operation: OperationDefinition) -> RequestPlan {
        let mut stack = vec![ToBePlanned {
            parent: None,
            path: ResponsePath::empty(),
            selection_set: operation.selection_set.clone(),
        }];
        while let Some(to_be_planned) = stack.pop() {
            stack.extend(self.plan_next(to_be_planned));
        }
        RequestPlan {
            operation_type: operation.ty,
            operation_selection_set: operation.selection_set,
            response_fields: self.response_fields.build(),
            execution_plans: self.plans.build(),
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

        #[derive(Default)]
        pub struct ResolverIO {
            input: schema::SelectionSet,
            output: Vec<FieldId>,
        }

        let mut candidates = HashMap::<ResolverId, ResolverIO>::new();
        for selection in &selection_set {
            let field_id = self.response_fields[selection.field].field_id;
            for FieldResolver { resolver_id, requires } in &self.engine.schema[field_id].resolvers {
                let io = candidates.entry(*resolver_id).or_default();
                io.input = schema::SelectionSet::merge(&io.input, requires);
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
            let (output, mut to_be_planned, rest) =
                self.partition_selection_set(resolver.data_source_id(), io.output, &path, selection_set);
            selection_set = rest;
            let plan_id = self.plans.push_plan(ExecutionPlan {
                path: path.clone(),
                input: ReadSelectionSet::empty(),
                output,
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

        assert!(children.is_empty());
        children
    }

    fn partition_selection_set(
        &self,
        data_source_id: DataSourceId,
        mut output: Vec<FieldId>,
        path: &ResponsePath,
        selection_set: SelectionSet,
    ) -> (WriteSelectionSet, Vec<ToBePlanned>, SelectionSet) {
        output.sort_unstable();
        let mut to_be_planned = vec![];

        let (output_node_selection_set, rest) = selection_set
            .into_iter()
            .map(|selection| {
                let edge = &self.response_fields[selection.field];
                if output.binary_search(&edge.field_id).is_ok() {
                    let field = &self.engine.schema[edge.field_id];

                    Ok(WriteSelection {
                        field: selection.field,
                        subselection: self.assign_provideable_output_node_selection_set(
                            data_source_id,
                            field.provides(data_source_id).cloned(),
                            true,
                            path.child(selection.field),
                            selection.subselection,
                            &mut to_be_planned,
                        ),
                    })
                } else {
                    Err(selection)
                }
            })
            .partition_result();

        (output_node_selection_set, to_be_planned, rest)
    }

    fn assign_provideable_output_node_selection_set(
        &self,
        data_source_id: DataSourceId,
        provideable: Option<schema::SelectionSet>,
        assign_without_resolvers: bool,
        path: ResponsePath,
        selection_set: SelectionSet,
        to_be_planned: &mut Vec<ToBePlanned>,
    ) -> WriteSelectionSet {
        let (output, missing): (WriteSelectionSet, SelectionSet) = selection_set
            .into_iter()
            .map(|selection| {
                let edge = &self.response_fields[selection.field];
                let field = &self.engine.schema[edge.field_id];

                let provideable_selection = provideable.as_ref().and_then(|s| s.selection(edge.field_id));
                if provideable_selection.is_some() || (assign_without_resolvers && field.resolvers.is_empty()) {
                    let parent_provideable = provideable_selection.map(|s| &s.subselection);
                    let current_provideable = field.provides(data_source_id);
                    let provideable: Option<schema::SelectionSet> = match (parent_provideable, current_provideable) {
                        (None, None) => None,
                        (Some(a), Some(b)) => Some(schema::SelectionSet::merge(a, b)),
                        (None, p) | (p, None) => p.cloned(),
                    };
                    Ok(WriteSelection {
                        field: selection.field,
                        subselection: self.assign_provideable_output_node_selection_set(
                            data_source_id,
                            provideable,
                            field.resolvers.is_empty(),
                            path.child(selection.field),
                            selection.subselection,
                            to_be_planned,
                        ),
                    })
                } else {
                    Err(selection)
                }
            })
            .partition_result();
        to_be_planned.push(ToBePlanned {
            parent: None, // defined later
            path,
            selection_set: missing,
        });
        output
    }

    // Here we need to ensure that the requires NodeSelectionSet uses existing fields when possible
    fn create_input(
        &mut self,
        output_set: &mut WriteSelectionSet,
        pos: Pos,
        type_condition: Option<TypeCondition>,
        required_selection_set: &schema::SelectionSet,
    ) -> ReadSelectionSet {
        required_selection_set
            .into_iter()
            .map(|required_selection| {
                let maybe_output = output_set.iter_mut().find_map(|output| {
                    let edge = &self.response_fields[output.field];
                    if edge.field_id == required_selection.field
                        && type_condition
                            .zip(edge.type_condition)
                            .map(|(a, b)| a == b)
                            .unwrap_or(edge.type_condition.is_none())
                        && edge.arguments.is_empty()
                    {
                        Some((edge.name, output))
                    } else {
                        None
                    }
                });
                match maybe_output {
                    Some((name, output_selection)) => ReadSelection {
                        name,
                        subselection: self.create_input(
                            &mut output_selection.subselection,
                            pos,
                            None,
                            &required_selection.subselection,
                        ),
                    },
                    None => {
                        let schema_field = &self.engine.schema[required_selection.field];
                        let (field, name) = self.response_fields.push_internal_field(
                            &self.engine.schema[schema_field.name],
                            pos,
                            required_selection.field,
                            None,
                            vec![],
                        );
                        let new_output = output_set.insert(WriteSelection {
                            field,
                            subselection: WriteSelectionSet::empty(),
                        });
                        let subselection = self.create_input(
                            &mut new_output.subselection,
                            pos,
                            None,
                            &required_selection.subselection,
                        );
                        ReadSelection { name, subselection }
                    }
                }
            })
            .collect()
    }
}

struct ToBePlanned {
    parent: Option<PlanId>,
    path: ResponsePath,
    selection_set: SelectionSet,
}
