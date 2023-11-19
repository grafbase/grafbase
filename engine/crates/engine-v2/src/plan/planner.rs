use std::collections::{HashMap, HashSet, VecDeque};

use itertools::Itertools;
use schema::{DataSourceId, FieldSet, ResolverId};

use super::{
    attribution::AttributionBuilder, plans::ExecutionPlansBuilder, Attribution, ExecutionPlan, ExecutionPlans,
    OperationPlan, PlanId,
};
use crate::{
    plan::SelectionSetRoot,
    request::{
        BoundFieldId, BoundSelectionSetWalker, FlattenedBoundField, Operation, QueryPath, QueryPathSegment,
        ResolvedTypeCondition,
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

fn create_final_read_selection_set(selection_set: BoundSelectionSetWalker<'_>) -> ReadSelectionSet {
    selection_set
        // TODO: Will be removed later, the response is already in the right format. We just need
        // to read the fields in the right order.
        .flatten_fields()
        .map(|field| ReadSelection {
            response_name: field.response_name(),
            subselection: create_final_read_selection_set(field.selection_set()),
        })
        .collect()
}

#[allow(clippy::unnecessary_wraps)]
pub(super) fn plan_operation(engine: &Engine, mut operation: Operation) -> PrepareResult<OperationPlan> {
    // Creating the final read selection set immediately before any modifications (plan inputs
    // adding fields)
    let final_read_selection_set =
        create_final_read_selection_set(operation.walk_root_selection_set(engine.schema.default_walker()));
    let attribution = Attribution::builder(&operation);
    let to_be_planned = VecDeque::from([ToBePlannedSelectionSet {
        parent: None,
        root: SelectionSetRoot {
            path: QueryPath::empty(),
            id: operation.root_selection_set_id,
        },
        fields: operation
            .walk_root_selection_set(engine.schema.default_walker())
            .flatten_fields()
            .map(Into::into)
            .collect(),
    }]);
    let mut planner = Planner {
        engine,
        operation: &mut operation,
        plans: ExecutionPlans::builder(),
        attribution,
        to_be_planned,
    };
    while let Some(to_be_planned) = planner.to_be_planned.pop_front() {
        planner.plan_next(to_be_planned);
    }

    let execution_plans = planner.plans.build();
    let attribution = planner.attribution.build(&engine.schema, &operation);
    Ok(OperationPlan {
        operation,
        execution_plans,
        attribution,
        final_read_selection_set,
    })
}

struct Planner<'a> {
    engine: &'a Engine,
    operation: &'a mut Operation,
    plans: ExecutionPlansBuilder,
    attribution: AttributionBuilder,
    to_be_planned: VecDeque<ToBePlannedSelectionSet>,
}

#[derive(Debug)]
struct ToBePlannedSelectionSet {
    parent: Option<PlanId>,
    root: SelectionSetRoot,
    fields: HashSet<ToBePlannedField>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct ToBePlannedField {
    resolved_type_condition: Option<ResolvedTypeCondition>,
    id: BoundFieldId,
}

impl From<FlattenedBoundField<'_>> for ToBePlannedField {
    fn from(field: FlattenedBoundField<'_>) -> Self {
        ToBePlannedField {
            resolved_type_condition: field.resolved_type_condition.clone(),
            id: field.bound_field_id(),
        }
    }
}

struct ToBeAssignedField {
    root: SelectionSetRoot,
    provideable: FieldSet,
    id: BoundFieldId,
    data_source_id: DataSourceId,
    plan_id: PlanId,
    continuous: bool,
}

impl<'a> Planner<'a> {
    fn plan_next(
        &mut self,
        ToBePlannedSelectionSet {
            parent,
            root,
            mut fields,
        }: ToBePlannedSelectionSet,
    ) {
        while !fields.is_empty() {
            pub struct Candidate {
                resolver_id: ResolverId,
                requires: FieldSet,
                output: HashSet<ToBePlannedField>,
            }

            let mut candidates = HashMap::<ResolverId, Candidate>::new();
            for field in &fields {
                for field_resolver in self
                    .operation
                    .walk_field(self.engine.schema.default_walker(), field.id)
                    .resolvers()
                {
                    let candidate = candidates
                        .entry(field_resolver.resolver_id)
                        .or_insert_with_key(|&resolver_id| Candidate {
                            resolver_id,
                            requires: FieldSet::default(),
                            output: HashSet::new(),
                        });
                    candidate.requires = schema::FieldSet::merge(&candidate.requires, &field_resolver.requires);
                    candidate.output.insert(field.clone());
                }
            }

            // We assume no inputs and separate outputs for now.
            // Later we could:
            // - Determine which candidate need additional data (mapping requires to actual fields or
            //   check whether they could be provided from parent/sibling plans).
            // - plan the one with most fields.
            let candidate = candidates.into_iter().next().unwrap().1;
            assert!(candidate.requires.is_empty());

            let plan_id = self.plans.push(ExecutionPlan {
                root: root.clone(),
                input: ReadSelectionSet::empty(),
                resolver_id: candidate.resolver_id,
            });
            if let Some(parent) = parent {
                self.plans.add_dependency(plan_id, parent);
            }

            let data_source_id = self.engine.schema[candidate.resolver_id].data_source_id();
            for field in candidate.output {
                fields.remove(&field);
                let resolved_type_condition = field.resolved_type_condition.clone();
                let field = self.operation.walk_field(self.engine.schema.default_walker(), field.id);
                self.assign_field(ToBeAssignedField {
                    root: SelectionSetRoot {
                        path: root.path.child(QueryPathSegment {
                            resolved_type_condition,
                            name: field.response_name(),
                        }),
                        id: field.selection_set().id,
                    },
                    provideable: field.provides(data_source_id).cloned().unwrap_or_default(),
                    id: field.bound_field_id(),
                    continuous: true,
                    data_source_id,
                    plan_id,
                });
            }
        }
    }

    fn assign_field(
        &mut self,
        ToBeAssignedField {
            root,
            provideable,
            id,
            data_source_id,
            plan_id,
            continuous,
        }: ToBeAssignedField,
    ) {
        self.attribution.attribute(id, plan_id);
        let walker = self.operation.walk_field(self.engine.schema.default_walker(), id);

        let (to_be_assigned, to_be_planned): (Vec<_>, HashSet<_>) = walker
            .selection_set()
            .flatten_fields()
            .map(|field| {
                let provideable_selection = provideable.get(field.id);
                if provideable_selection.is_some() || (continuous && field.resolvers.is_empty()) {
                    Ok(ToBeAssignedField {
                        root: SelectionSetRoot {
                            path: root.path.child(QueryPathSegment {
                                resolved_type_condition: field.resolved_type_condition.clone(),
                                name: field.response_name(),
                            }),
                            id: field.selection_set().id,
                        },
                        provideable: FieldSet::merge_opt(Some(&provideable), field.provides(data_source_id)),
                        id: field.bound_field_id(),
                        continuous: field.resolvers.is_empty(),
                        data_source_id,
                        plan_id,
                    })
                } else {
                    Err(ToBePlannedField::from(field))
                }
            })
            .partition_result();

        for field in to_be_assigned {
            self.assign_field(field);
        }

        if !to_be_planned.is_empty() {
            self.to_be_planned.push_back(ToBePlannedSelectionSet {
                parent: Some(plan_id),
                root,
                fields: to_be_planned,
            });
        }
    }
}
