use std::collections::HashMap;

use engine_parser::{types::OperationType, Pos};
use itertools::Itertools;
use schema::{DataSourceId, FieldId, FieldResolver, ResolverId, SelectionSet, Translator};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use super::{ExecutionPlan, ExecutionPlanGraph, PlanId};
use crate::{
    request::OperationDefinition,
    response_graph::{
        FieldName, InputNodeSelection, InputNodeSelectionSet, NodePath, NodeSelectionSet, OutputNodeSelection,
        OutputNodeSelectionSet, ResponseGraphEdges, ResponseGraphEdgesBuilder, TypeCondition,
    },
    Engine,
};

new_key_type! { struct PlanningId; }

// This is the part that should be cached for a GraphQL query.
// I suppose we might want to plan all query operations together if we support operation batching.
// we would just need to keep track of the operation name to the selection_set.
// If that doesn't make any sense, we should rename that to OperationPlan
pub struct RequestPlan {
    pub operation_type: OperationType,
    pub operation_selection_set: NodeSelectionSet,
    pub response_graph_edges: ResponseGraphEdges,
    pub execution_plan_graph: ExecutionPlanGraph,
}

impl RequestPlan {
    pub fn builder(engine: &Engine) -> RequestPlanBuilder<'_> {
        RequestPlanBuilder {
            engine,
            plans: SlotMap::with_key(),
            reponse_graph_edges_builder: ResponseGraphEdges::builder(),
            parent_to_children: SecondaryMap::new(),
        }
    }
}

pub struct RequestPlanBuilder<'a> {
    engine: &'a Engine,
    reponse_graph_edges_builder: ResponseGraphEdgesBuilder,
    // Currently, we could use ExecutionPlanGraph directly as I'm not deleting any plans. But I
    // didn't start with that. Initially I intended to merge plans together. As it's not obvious
    // for now wether we'll do it later or not for now, keeping the current structure which
    // supports deletions.
    // On second thought we probably should use ExecutionPlanGraph directly for performance at
    // runtime.
    plans: SlotMap<PlanningId, ExecutionPlan>,                     // nodes
    parent_to_children: SecondaryMap<PlanningId, Vec<PlanningId>>, // outgoing edges
}

impl<'a> RequestPlanBuilder<'a> {
    pub fn build(mut self, operation: OperationDefinition) -> RequestPlan {
        let mut stack = vec![ToBePlanned {
            parent: None,
            path: NodePath::empty(),
            selection_set: operation.selection_set.clone(),
        }];
        while let Some(to_be_planned) = stack.pop() {
            stack.extend(self.plan_next(to_be_planned));
        }
        let mut graph_builder = ExecutionPlanGraph::builder();
        let mut translation = HashMap::<PlanningId, PlanId>::new();
        // Not guaranteed to be kept when remove keys from plans as far as I understood. Need to
        // read how it the secondary map is implemented. ^^
        let parent_to_children = self.parent_to_children.into_iter().collect::<HashMap<_, _>>();
        for (plannin_id, plan) in self.plans {
            let plan_id = graph_builder.push_plan(plan);
            translation.insert(plannin_id, plan_id);
        }

        for (parent, children) in parent_to_children {
            for child in children {
                graph_builder.push_dependency(translation[&parent], translation[&child]);
            }
        }
        RequestPlan {
            operation_type: operation.ty,
            operation_selection_set: operation.selection_set,
            response_graph_edges: self.reponse_graph_edges_builder.build(),
            execution_plan_graph: graph_builder.build(),
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
            input: SelectionSet,
            output: Vec<FieldId>,
        }

        let mut candidates = HashMap::<ResolverId, ResolverIO>::new();
        for selection in &selection_set {
            let field_id = self.reponse_graph_edges_builder[selection.field].field_id;
            for FieldResolver { resolver_id, requires } in &self.engine.schema[field_id].resolvers {
                let io = candidates.entry(*resolver_id).or_default();
                io.input = SelectionSet::merge(&io.input, requires);
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
            let plan_id = self.plans.insert(ExecutionPlan {
                path: path.clone(),
                input: InputNodeSelectionSet::empty(),
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
            for plan_id in &plan_ids {
                self.add_child_edge(parent, *plan_id);
            }
        }

        assert!(children.is_empty());
        children
    }

    fn add_child_edge(&mut self, parent: PlanningId, child: PlanningId) {
        self.parent_to_children.entry(parent).unwrap().or_default().push(child);
    }

    fn partition_selection_set(
        &self,
        data_source_id: DataSourceId,
        mut output: Vec<FieldId>,
        path: &NodePath,
        selection_set: NodeSelectionSet,
    ) -> (OutputNodeSelectionSet, Vec<ToBePlanned>, NodeSelectionSet) {
        output.sort_unstable();
        let mut to_be_planned = vec![];

        let (output_node_selection_set, rest) = selection_set
            .into_iter()
            .map(|selection| {
                let edge = &self.reponse_graph_edges_builder[selection.field];
                if output.binary_search(&edge.field_id).is_ok() {
                    let field = &self.engine.schema[edge.field_id];

                    Ok(OutputNodeSelection {
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
        provideable: Option<SelectionSet>,
        assign_without_resolvers: bool,
        path: NodePath,
        selection_set: NodeSelectionSet,
        to_be_planned: &mut Vec<ToBePlanned>,
    ) -> OutputNodeSelectionSet {
        let (output, missing): (OutputNodeSelectionSet, NodeSelectionSet) = selection_set
            .into_iter()
            .map(|selection| {
                let edge = &self.reponse_graph_edges_builder[selection.field];
                let field = &self.engine.schema[edge.field_id];

                let provideable_selection = provideable.as_ref().and_then(|s| s.selection(edge.field_id));
                if provideable_selection.is_some() || (assign_without_resolvers && field.resolvers.is_empty()) {
                    let parent_provideable = provideable_selection.map(|s| &s.subselection);
                    let current_provideable = field.provides(data_source_id);
                    let provideable: Option<SelectionSet> = match (parent_provideable, current_provideable) {
                        (None, None) => None,
                        (Some(a), Some(b)) => Some(SelectionSet::merge(a, b)),
                        (None, p) | (p, None) => p.cloned(),
                    };
                    Ok(OutputNodeSelection {
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
        output_set: &mut OutputNodeSelectionSet,
        pos: Pos,
        type_condition: Option<TypeCondition>,
        translator: &dyn Translator,
        input_set: &SelectionSet,
    ) -> InputNodeSelectionSet {
        input_set
            .into_iter()
            .map(|input| {
                let maybe_output = output_set.iter_mut().find_map(|output| {
                    let edge = &self.reponse_graph_edges_builder[output.field];
                    if edge.field_id == input.field
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
                let input_name: FieldName = self
                    .reponse_graph_edges_builder
                    .intern_field_name(translator.field(input.field));
                match maybe_output {
                    Some((name, output_selection)) => InputNodeSelection {
                        field: output_selection.field,
                        name,
                        input_name,
                        subselection: self.create_input(
                            &mut output_selection.subselection,
                            pos,
                            None,
                            translator,
                            &input.subselection,
                        ),
                    },
                    None => {
                        let schema_field = &self.engine.schema[input.field];
                        let (field, name) = self.reponse_graph_edges_builder.push_internal_field(
                            &self.engine.schema[schema_field.name],
                            pos,
                            input.field,
                            None,
                            vec![],
                        );
                        let new_output = output_set.insert(OutputNodeSelection {
                            field,
                            subselection: OutputNodeSelectionSet::empty(),
                        });
                        let subselection =
                            self.create_input(&mut new_output.subselection, pos, None, translator, &input.subselection);
                        InputNodeSelection {
                            field,
                            name,
                            input_name,
                            subselection,
                        }
                    }
                }
            })
            .collect()
    }
}

struct ToBePlanned {
    parent: Option<PlanningId>,
    path: NodePath,
    selection_set: NodeSelectionSet,
}
