mod field;
mod field_argument;
mod flat;
mod fragment;
mod inline_fragment;
mod operation_limits;
mod plan;
mod plan_selection_set;
mod selection_set;
mod variables;

use std::{
    borrow::Cow,
    collections::{HashSet, VecDeque},
};

pub use field::*;
pub use field_argument::*;
pub use flat::*;
pub use fragment::*;
pub use inline_fragment::*;
pub use plan::*;
pub use plan_selection_set::*;
use schema::{ObjectId, Schema, SchemaWalker};
pub use selection_set::*;
pub use variables::*;

use crate::request::SelectionSetType;

use super::{
    BoundFieldId, BoundSelection, BoundSelectionSetId, FlatField, FlatSelectionSet, FlatSelectionSetId,
    FlatTypeCondition, Operation, TypeCondition,
};

#[derive(Clone, Copy)]
pub struct OperationWalker<'a, Item = (), SchemaItem = (), ExecutorWalkContextOrUnit = ()> {
    /// The operation MUST NOT be accessible directly in any form. Plans may have additional
    /// internal fields which we can't add to the operation easily. It's shared during execution.
    /// Those internal fields must present themselves like real proper operation fields for plan
    /// which direct access to the Operation can't do obviously.
    pub(super) operation: &'a Operation,
    pub(super) schema_walker: SchemaWalker<'a, SchemaItem>,
    /// Walkers are used in two different situations:
    /// - during planning
    /// - by executors, with plan & execution metadata: attributed fields and variables.
    /// In practice this type is a `() | ExecutorWalkContext` as some methods only make sense in either case
    /// and others are common.
    pub(super) ctx: ExecutorWalkContextOrUnit,
    pub(super) item: Item,
}

impl<'a> std::fmt::Debug for OperationWalker<'a, (), (), ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationWalker").finish_non_exhaustive()
    }
}

impl<'a, I: Copy, SI, C> OperationWalker<'a, I, SI, C>
where
    Operation: std::ops::Index<I>,
{
    pub fn as_ref(&self) -> &'a <Operation as std::ops::Index<I>>::Output {
        &self.operation[self.item]
    }

    pub fn id(&self) -> I {
        self.item
    }
}

impl<'a, I, SI: Copy, C> OperationWalker<'a, I, SI, C>
where
    Schema: std::ops::Index<SI>,
{
    pub fn schema_id(&self) -> SI {
        self.schema_walker.id()
    }
}

impl<'a, C> OperationWalker<'a, (), (), C> {
    pub fn schema(&self) -> SchemaWalker<'a, ()> {
        self.schema_walker
    }

    pub fn names(&self) -> &'a dyn schema::Names {
        self.schema_walker.names()
    }
}

impl<'a, I, SI, C> OperationWalker<'a, I, SI, C> {
    pub fn walk<I2>(&self, item: I2) -> OperationWalker<'a, I2, SI, C>
    where
        SI: Copy,
        C: Copy,
    {
        OperationWalker {
            operation: self.operation,
            schema_walker: self.schema_walker,
            ctx: self.ctx,
            item,
        }
    }

    pub fn walk_with<I2, SI2>(&self, item: I2, schema_item: SI2) -> OperationWalker<'a, I2, SI2, C>
    where
        C: Copy,
    {
        OperationWalker {
            operation: self.operation,
            schema_walker: self.schema_walker.walk(schema_item),
            ctx: self.ctx,
            item,
        }
    }

    pub fn with_ctx(&self, ctx: ExecutorWalkContext<'a>) -> OperationWalker<'a, I, SI, ExecutorWalkContext<'a>>
    where
        I: Copy,
        SI: Copy,
    {
        OperationWalker {
            operation: self.operation,
            schema_walker: self.schema_walker,
            ctx,
            item: self.item,
        }
    }

    pub fn without_ctx(&self) -> OperationWalker<'a, I, SI, ()>
    where
        I: Copy,
        SI: Copy,
    {
        OperationWalker {
            operation: self.operation,
            schema_walker: self.schema_walker,
            ctx: (),
            item: self.item,
        }
    }
}

impl<'a> OperationWalker<'a> {
    pub fn root_object_id(&self) -> ObjectId {
        self.operation.root_object_id
    }

    pub fn merged_selection_sets(&self, bound_field_ids: &[BoundFieldId]) -> FlatSelectionSetWalker<'a, 'static> {
        self.flatten_selection_sets(
            bound_field_ids
                .iter()
                .filter_map(|id| self.operation[*id].selection_set_id())
                .collect(),
        )
    }

    pub fn flatten_selection_sets(
        &self,
        merged_selection_set_ids: Vec<BoundSelectionSetId>,
    ) -> FlatSelectionSetWalker<'a, 'static> {
        let id = FlatSelectionSetId::from(merged_selection_set_ids[0]);
        let selection_set_type = {
            let ty = merged_selection_set_ids
                .iter()
                .map(|id| self.operation[*id].ty)
                .collect::<HashSet<SelectionSetType>>();
            assert_eq!(ty.len(), 1);
            ty.into_iter().next().unwrap()
        };
        let mut fields = Vec::new();
        let mut selections = VecDeque::from_iter(merged_selection_set_ids.into_iter().flat_map(|selection_set_id| {
            self.operation[selection_set_id]
                .items
                .iter()
                .map(move |selection| (Vec::<TypeCondition>::new(), vec![selection_set_id], selection))
        }));
        while let Some((mut type_condition_chain, mut selection_set_path, selection)) = selections.pop_front() {
            match selection {
                &BoundSelection::Field(bound_field_id) => {
                    let type_condition =
                        FlatTypeCondition::flatten(&self.schema_walker, selection_set_type, type_condition_chain);
                    if FlatTypeCondition::is_possible(&type_condition) {
                        fields.push(FlatField {
                            type_condition,
                            selection_set_path,
                            bound_field_id,
                        });
                    }
                }
                BoundSelection::FragmentSpread(spread_id) => {
                    let spread = &self.operation[*spread_id];
                    let fragment = &self.operation[spread.fragment_id];
                    type_condition_chain.push(fragment.type_condition);
                    selection_set_path.push(spread.selection_set_id);
                    selections.extend(
                        self.operation[spread.selection_set_id]
                            .items
                            .iter()
                            .map(|selection| (type_condition_chain.clone(), selection_set_path.clone(), selection)),
                    );
                }
                BoundSelection::InlineFragment(inline_fragment_id) => {
                    let inline_fragment = &self.operation[*inline_fragment_id];
                    if let Some(type_condition) = inline_fragment.type_condition {
                        type_condition_chain.push(type_condition);
                    }
                    selection_set_path.push(inline_fragment.selection_set_id);
                    selections.extend(
                        self.operation[inline_fragment.selection_set_id]
                            .items
                            .iter()
                            .map(|selection| (type_condition_chain.clone(), selection_set_path.clone(), selection)),
                    );
                }
            }
        }

        self.walk(Cow::Owned(FlatSelectionSet {
            id,
            ty: selection_set_type,
            fields,
        }))
    }
}

fn type_condition_name<I>(schema: SchemaWalker<'_, I>, type_condition: TypeCondition) -> &str {
    match type_condition {
        TypeCondition::Interface(interface_id) => schema.walk(interface_id).name(),
        TypeCondition::Object(object_id) => schema.walk(object_id).name(),
        TypeCondition::Union(union_id) => schema.walk(union_id).name(),
    }
}
