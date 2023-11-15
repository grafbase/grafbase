use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
};

use itertools::Itertools;
use schema::{DataSourceId, FieldId, FieldResolver, ResolverId};

use super::{plans::ExecutionPlansBuilder, ExecutionPlan, ExecutionPlans, PlanId};
use crate::{
    execution::ExecutionStrings,
    formatter::{ContextAwareDebug, FormatterContext, FormatterContextHolder},
    request::{
        Operation, OperationFieldId, OperationFields, OperationFieldsBuilder, OperationPath, OperationPathSegment,
        OperationSelectionSet, OperationType, ResolvedTypeCondition, TypeCondition,
    },
    response::{ReadSelection, ReadSelectionSet, WriteSelection, WriteSelectionSet},
    Engine,
};

// This is the part that should be cached for a GraphQL query.
// I suppose we might want to plan all query operations together if we support operation batching.
// we would just need to keep track of the operation name to the selection_set.
// If that doesn't make any sense, we should rename that to OperationPlan
pub struct PlannedOperation {
    pub ty: OperationType,
    pub selection_set: ReadSelectionSet,
    pub fields: OperationFields,
    pub strings: ExecutionStrings,
    pub plans: ExecutionPlans,
}

impl PlannedOperation {
    pub fn build(engine: &Engine, operation: Operation) -> PlannedOperation {
        let Operation {
            ty,
            selection_set,
            fields,
            mut strings,
        } = operation;

        let mut planner = Planner {
            engine,
            plans: ExecutionPlans::builder(),
            fields: fields.into_builder(&mut strings),
        };
        planner.plan_operation(&selection_set);
        let selection_set = planner.create_final_read_selection_set(&selection_set);
        let fields = planner.fields.build();
        let execution_plans = planner.plans.build();

        PlannedOperation {
            ty,
            selection_set,
            fields,
            strings,
            plans: execution_plans,
        }
    }
}

pub struct Planner<'a> {
    engine: &'a Engine,
    fields: OperationFieldsBuilder<'a>,
    plans: ExecutionPlansBuilder,
}

impl<'a> Planner<'a> {
    fn create_final_read_selection_set(&self, selection_set: &OperationSelectionSet) -> ReadSelectionSet {
        selection_set
            .iter()
            .map(|selection| {
                let op_field = &self.fields[selection.op_field_id];
                ReadSelection {
                    response_position: op_field.position,
                    response_name: op_field.name,
                    subselection: self.create_final_read_selection_set(&selection.subselection),
                }
            })
            .collect()
    }

    fn plan_operation(&mut self, selection_set: &OperationSelectionSet) {
        let mut stack = vec![ToBePlanned {
            parent: None,
            path: OperationPath::empty(),
            selection_set: selection_set.clone(),
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
            let field_id = self.fields[selection.op_field_id].field_id;
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
        // will need to reworked with inputs adding fields.
        let dense_capacity = self.compute_dense_capacity(selection_set.iter().map(|selection| selection.op_field_id));
        for (resolver_id, io) in candidates {
            assert!(io.input.is_empty());
            let resolver = &self.engine.schema[resolver_id];
            let (write_selections, mut to_be_planned, rest) =
                self.partition_selection_set(resolver.data_source_id(), io.output, &path, selection_set);
            selection_set = rest;

            let plan_id = self.plans.push(ExecutionPlan {
                path: path.clone(),
                input: ReadSelectionSet::empty(),
                output: WriteSelectionSet::new(dense_capacity, write_selections),
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

        assert!(children.is_empty(), "{children:?}");
        children
    }

    fn compute_dense_capacity(&self, fields: impl IntoIterator<Item = OperationFieldId>) -> Option<NonZeroUsize> {
        let mut count: usize = 0;
        let mut type_conditions = HashSet::new();
        for id in fields {
            count += 1;
            type_conditions.insert(self.fields[id].type_condition);
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
    ) -> (Vec<WriteSelection>, Vec<ToBePlanned>, OperationSelectionSet) {
        output_fields.sort_unstable();
        let mut to_be_planned = vec![];

        let (write_selections, rest) = selection_set
            .into_iter()
            .map(|selection| {
                let op_field = &self.fields[selection.op_field_id];
                if output_fields.binary_search(&op_field.field_id).is_ok() {
                    let schema_field = &self.engine.schema[op_field.field_id];

                    Ok(WriteSelection {
                        operation_field_id: selection.op_field_id,
                        response_position: op_field.position,
                        response_name: op_field.name,
                        subselection: self.assign_provideable_write_selection_set(
                            data_source_id,
                            schema_field.provides(data_source_id).cloned().unwrap_or_default(),
                            true,
                            self.make_path_child(path, selection.op_field_id),
                            selection.subselection,
                            &mut to_be_planned,
                        ),
                    })
                } else {
                    Err(selection)
                }
            })
            .partition_result();

        (write_selections, to_be_planned, rest)
    }

    fn assign_provideable_write_selection_set(
        &self,
        data_source_id: DataSourceId,
        provideable: schema::FieldSet,
        assign_without_resolvers: bool,
        path: OperationPath,
        selection_set: OperationSelectionSet,
        to_be_planned: &mut Vec<ToBePlanned>,
    ) -> WriteSelectionSet {
        let dense_capacity = self.compute_dense_capacity(selection_set.iter().map(|selection| selection.op_field_id));
        let (items, missing): (Vec<WriteSelection>, OperationSelectionSet) = selection_set
            .into_iter()
            .map(|selection| {
                let op_field = &self.fields[selection.op_field_id];
                let schema_field = &self.engine.schema[op_field.field_id];

                let provideable_field = provideable.selection(op_field.field_id);
                if provideable_field.is_some() || (assign_without_resolvers && schema_field.resolvers.is_empty()) {
                    let provideable = schema::FieldSet::merge_opt(
                        provideable_field.map(|s| &s.subselection),
                        schema_field.provides(data_source_id),
                    );
                    Ok(WriteSelection {
                        operation_field_id: selection.op_field_id,
                        response_position: op_field.position,
                        response_name: op_field.name,
                        subselection: self.assign_provideable_write_selection_set(
                            data_source_id,
                            provideable,
                            schema_field.resolvers.is_empty(),
                            self.make_path_child(&path, selection.op_field_id),
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
        WriteSelectionSet::new(dense_capacity, items)
    }

    fn make_path_child(&self, parent: &OperationPath, child: OperationFieldId) -> OperationPath {
        let reponse_field = &self.fields[child];
        parent.child(OperationPathSegment {
            operation_field_id: child,
            type_condition: reponse_field.type_condition.map(|cond| {
                ResolvedTypeCondition::new(match cond {
                    TypeCondition::Interface(interface_id) => self.engine.schema[interface_id].implementations.clone(),
                    TypeCondition::Object(object_id) => vec![object_id],
                    TypeCondition::Union(union_id) => self.engine.schema[union_id].members.clone(),
                })
            }),
            position: reponse_field.position,
            name: reponse_field.name,
        })
    }

    // // Here we need to ensure that the requires NodeSelectionSet uses existing fields when possible
    // fn create_input(
    //     &mut self,
    //     parent_output_set: &mut WriteSelectionSet,
    //     pos: Pos,
    //     type_condition: Option<TypeCondition>,
    //     required_selection_set: &schema::FieldSet,
    // ) -> ReadSelectionSet {
    //     required_selection_set
    //         .into_iter()
    //         .map(|required_selection| {
    //             let maybe_existing_selection = parent_output_set.iter_mut().find_map(|output| {
    //                 let response_field = &self.response_fields[output.field];
    //                 if response_field.field_id == required_selection.field
    //                     && type_condition
    //                         .zip(response_field.type_condition)
    //                         .map(|(a, b)| a == b)
    //                         .unwrap_or(response_field.type_condition.is_none())
    //                     && response_field.arguments.is_empty()
    //                 {
    //                     Some((response_field, output))
    //                 } else {
    //                     None
    //                 }
    //             });
    //             match maybe_existing_selection {
    //                 Some((response_field, output_selection)) => ReadSelection {
    //                     name: response_field.name,
    //                     position: response_field.position,
    //                     subselection: self.create_input(
    //                         &mut output_selection.subselection,
    //                         pos,
    //                         None,
    //                         &required_selection.subselection,
    //                     ),
    //                 },
    //                 None => {
    //                     let schema_field = &self.engine.schema[required_selection.field];
    //                     let (field, name) = self.response_fields.push_internal_field(
    //                         &self.engine.schema[schema_field.name],
    //                         pos,
    //                         required_selection.field,
    //                         None,
    //                         vec![],
    //                     );
    //                     let new_output = parent_output_set.insert_internal(WriteSelection {
    //                         field,
    //                         subselection: WriteSelectionSet::empty(),
    //                     });
    //                     let subselection = self.create_input(
    //                         &mut new_output.subselection,
    //                         pos,
    //                         None,
    //                         &required_selection.subselection,
    //                     );
    //                     ReadSelection { name, subselection }
    //                 }
    //             }
    //         })
    //         .collect()
    // }
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
            strings: self.fields.strings(),
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
