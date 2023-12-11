mod field;
mod field_argument;
mod field_definition;
mod flat;
mod fragment;
mod inline_fragment;
mod plan;
mod selection_set;
mod variables;

use std::{
    borrow::Cow,
    collections::{HashSet, VecDeque},
};

pub use field::*;
pub use field_argument::*;
pub use field_definition::*;
pub use flat::*;
pub use fragment::*;
pub use inline_fragment::*;
pub use plan::*;
use schema::SchemaWalker;
pub use selection_set::*;
pub use variables::*;

use crate::request::SelectionSetType;

use super::{
    BoundFieldId, BoundSelection, BoundSelectionSetId, FlatField, FlatSelectionSet, FlatTypeCondition, Operation,
    TypeCondition,
};

#[derive(Clone, Copy)]
pub struct OperationWalker<'a, Walkable = (), SchemaId = (), Extension = ()> {
    /// The operation MUST NOT be accessible directly in any form. Elans may have additional
    /// internal fields which we can't add to the operation easily. It's shared during execution.
    /// Those internal fields must present themselves like real proper operation fields for plan
    /// which direct access to the Operation can't do obviously.
    pub(super) operation: &'a Operation,
    pub(super) schema_walker: SchemaWalker<'a, SchemaId>,
    pub(super) ext: Extension,
    pub(super) wrapped: Walkable,
}

impl<'a> std::fmt::Debug for OperationWalker<'a, (), (), ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationWalker").finish_non_exhaustive()
    }
}

impl<'a, W: Copy, I, E> OperationWalker<'a, W, I, E>
where
    Operation: std::ops::Index<W>,
{
    pub fn get(&self) -> &'a <Operation as std::ops::Index<W>>::Output {
        &self.operation[self.wrapped]
    }

    pub fn id(&self) -> W {
        self.wrapped
    }
}

impl<'a, W: Copy, I, E> std::ops::Deref for OperationWalker<'a, W, I, E>
where
    Operation: std::ops::Index<W>,
{
    type Target = <Operation as std::ops::Index<W>>::Output;

    fn deref(&self) -> &Self::Target {
        &self.operation[self.wrapped]
    }
}

impl<'a, E> OperationWalker<'a, (), (), E> {
    pub fn schema(&self) -> SchemaWalker<'a, ()> {
        self.schema_walker
    }

    pub fn names(&self) -> &'a dyn schema::Names {
        self.schema_walker.names()
    }
}

impl<'a, W, I, E> OperationWalker<'a, W, I, E> {
    pub fn walk<W2>(&self, inner: W2) -> OperationWalker<'a, W2, I, E>
    where
        I: Copy,
        E: Copy,
    {
        OperationWalker {
            operation: self.operation,
            schema_walker: self.schema_walker,
            ext: self.ext,
            wrapped: inner,
        }
    }

    pub fn walk_with<W2, I2>(&self, inner: W2, schema_id: I2) -> OperationWalker<'a, W2, I2, E>
    where
        I: Copy,
        E: Copy,
    {
        OperationWalker {
            operation: self.operation,
            schema_walker: self.schema_walker.walk(schema_id),
            ext: self.ext,
            wrapped: inner,
        }
    }

    pub fn with_plan<E2>(&self, plan: E2) -> OperationWalker<'a, W, I, E2>
    where
        W: Copy,
        I: Copy,
    {
        OperationWalker {
            operation: self.operation,
            schema_walker: self.schema_walker,
            ext: plan,
            wrapped: self.wrapped,
        }
    }
}

impl<'a> OperationWalker<'a> {
    pub fn merged_selection_sets(&self, bound_field_ids: &[BoundFieldId]) -> FlatSelectionSetWalker<'a> {
        self.flatten_selection_sets(
            bound_field_ids
                .iter()
                .filter_map(|id| self.operation[*id].selection_set_id)
                .collect(),
        )
    }

    pub fn flatten_selection_sets(
        &self,
        merged_selection_set_ids: Vec<BoundSelectionSetId>,
    ) -> FlatSelectionSetWalker<'a> {
        let any_selection_set_id = merged_selection_set_ids[0];
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
                BoundSelection::FragmentSpread(spread) => {
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
                BoundSelection::InlineFragment(fragment) => {
                    if let Some(type_condition) = fragment.type_condition {
                        type_condition_chain.push(type_condition);
                    }
                    selection_set_path.push(fragment.selection_set_id);
                    selections.extend(
                        self.operation[fragment.selection_set_id]
                            .items
                            .iter()
                            .map(|selection| (type_condition_chain.clone(), selection_set_path.clone(), selection)),
                    );
                }
            }
        }

        self.walk(Cow::Owned(FlatSelectionSet {
            any_selection_set_id,
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
