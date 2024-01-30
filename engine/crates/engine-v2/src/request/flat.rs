use std::{borrow::Borrow, cmp::Ordering, collections::VecDeque};

use fnv::FnvHashSet;
use itertools::Itertools;
use schema::{Definition, InterfaceId, ObjectId, Schema};

use crate::request::{BoundFieldId, BoundSelectionSetId, SelectionSetType, TypeCondition};

use super::{BoundSelection, OperationWalker};

impl<'a> OperationWalker<'a> {
    pub fn flatten_subselection_sets<Item: Borrow<BoundFieldId>>(
        &self,
        items: impl IntoIterator<Item = Item>,
    ) -> Option<FlatSelectionSet> {
        let selection_set_ids = items
            .into_iter()
            .filter_map(|item| self.operation[*item.borrow()].selection_set_id())
            .collect::<Vec<_>>();
        if selection_set_ids.is_empty() {
            None
        } else {
            Some(self.flatten_selection_sets(selection_set_ids))
        }
    }

    pub fn flatten_selection_sets(&self, root_selection_set_ids: Vec<BoundSelectionSetId>) -> FlatSelectionSet {
        let ty = {
            let selection_set_types = root_selection_set_ids
                .iter()
                .map(|id| self.operation[*id].ty)
                .collect::<FnvHashSet<SelectionSetType>>();
            assert_eq!(
                selection_set_types.len(),
                1,
                "{}",
                selection_set_types
                    .into_iter()
                    .map(|ty| self.schema_walker.walk(Definition::from(ty)).name())
                    .join(", ")
            );
            selection_set_types.into_iter().next().unwrap()
        };
        let mut flat_selection_set = FlatSelectionSet {
            root_selection_set_ids,
            ty,
            fields: Vec::new(),
        };
        let mut selections = VecDeque::from_iter(flat_selection_set.root_selection_set_ids.iter().flat_map(
            |&selection_set_id| {
                self.operation[selection_set_id]
                    .items
                    .iter()
                    .map(move |selection| (Vec::<TypeCondition>::new(), vec![selection_set_id], selection))
            },
        ));
        while let Some((mut type_condition_chain, mut selection_set_path, selection)) = selections.pop_front() {
            match selection {
                &BoundSelection::Field(bound_field_id) => {
                    let type_condition =
                        FlatTypeCondition::flatten(&self.schema_walker, flat_selection_set.ty, type_condition_chain);
                    if FlatTypeCondition::is_possible(&type_condition) {
                        flat_selection_set.fields.push(FlatField {
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

        flat_selection_set
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FlatSelectionSet {
    pub ty: SelectionSetType,
    pub root_selection_set_ids: Vec<BoundSelectionSetId>,
    pub fields: Vec<FlatField>,
}

impl FlatSelectionSet {
    pub fn partition_fields(self, predicate: impl Fn(&FlatField) -> bool) -> (Self, Self) {
        let (left, right) = self.fields.into_iter().partition(predicate);
        (
            Self {
                ty: self.ty,
                root_selection_set_ids: self.root_selection_set_ids.clone(),
                fields: left,
            },
            Self { fields: right, ..self },
        )
    }

    pub fn clone_with_fields(&self, fields: Vec<FlatField>) -> Self {
        Self {
            ty: self.ty,
            root_selection_set_ids: self.root_selection_set_ids.clone(),
            fields,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl IntoIterator for FlatSelectionSet {
    type Item = FlatField;

    type IntoIter = <Vec<FlatField> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl<'a> IntoIterator for &'a FlatSelectionSet {
    type Item = &'a FlatField;

    type IntoIter = <&'a Vec<FlatField> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.iter()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FlatField {
    pub type_condition: Option<FlatTypeCondition>,
    // There is always at least one element.
    pub selection_set_path: Vec<BoundSelectionSetId>,
    pub bound_field_id: BoundFieldId,
}

impl Borrow<BoundFieldId> for FlatField {
    fn borrow(&self) -> &BoundFieldId {
        &self.bound_field_id
    }
}
impl Borrow<BoundFieldId> for &FlatField {
    fn borrow(&self) -> &BoundFieldId {
        &self.bound_field_id
    }
}

impl FlatField {
    pub fn parent_selection_set_id(&self) -> BoundSelectionSetId {
        self.selection_set_path.last().copied().unwrap()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntityType {
    Interface(InterfaceId),
    Object(ObjectId),
}

impl From<EntityType> for Definition {
    fn from(value: EntityType) -> Self {
        match value {
            EntityType::Interface(id) => Definition::Interface(id),
            EntityType::Object(id) => Definition::Object(id),
        }
    }
}

impl From<EntityType> for SelectionSetType {
    fn from(value: EntityType) -> Self {
        match value {
            EntityType::Interface(id) => SelectionSetType::Interface(id),
            EntityType::Object(id) => SelectionSetType::Object(id),
        }
    }
}

impl From<EntityType> for TypeCondition {
    fn from(value: EntityType) -> Self {
        match value {
            EntityType::Interface(id) => TypeCondition::Interface(id),
            EntityType::Object(id) => TypeCondition::Object(id),
        }
    }
}

impl EntityType {
    pub fn maybe_from(definition: Definition) -> Option<EntityType> {
        match definition {
            Definition::Object(id) => Some(EntityType::Object(id)),
            Definition::Interface(id) => Some(EntityType::Interface(id)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlatTypeCondition {
    Interface(InterfaceId),
    // sorted by ObjectId
    Objects(Box<[ObjectId]>),
}

impl FlatTypeCondition {
    pub fn is_possible(condition: &Option<Self>) -> bool {
        if let Some(condition) = condition {
            match condition {
                FlatTypeCondition::Interface(_) => true,
                FlatTypeCondition::Objects(ids) => !ids.is_empty(),
            }
        } else {
            true
        }
    }

    pub fn matches(&self, schema: &Schema, object_id: ObjectId) -> bool {
        match self {
            FlatTypeCondition::Interface(id) => schema[object_id].interfaces.contains(id),
            FlatTypeCondition::Objects(ids) => ids.binary_search(&object_id).is_ok(),
        }
    }

    pub fn flatten(schema: &Schema, ty: SelectionSetType, type_condition_chain: Vec<TypeCondition>) -> Option<Self> {
        let mut type_condition_chain = type_condition_chain.into_iter().peekable();
        let mut candidate = match ty {
            SelectionSetType::Object(object_id) => {
                // Checking that all type conditions apply.
                for type_condition in type_condition_chain {
                    match type_condition {
                        TypeCondition::Interface(id) if schema[object_id].interfaces.contains(&id) => (),
                        TypeCondition::Object(id) if object_id == id => (),
                        TypeCondition::Union(id) if schema[id].possible_types.contains(&object_id) => (),
                        _ => return Some(FlatTypeCondition::Objects(Box::new([]))),
                    }
                }
                return None;
            }
            SelectionSetType::Interface(id) => {
                // Any type condition that just applies on the root interface is ignored.
                while let Some(next) = type_condition_chain.peek().copied() {
                    if let TypeCondition::Interface(interface_id) = next {
                        if interface_id == id {
                            type_condition_chain.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                // If there no type conditions anymore it means it just applies to the root
                // directly.
                type_condition_chain.peek()?;
                FlatTypeCondition::Interface(id)
            }
            SelectionSetType::Union(union_id) => {
                // Any type condition that just applies on the root union is ignored.
                while let Some(next) = type_condition_chain.peek().copied() {
                    if let TypeCondition::Union(id) = next {
                        if union_id == id {
                            type_condition_chain.next();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                let first = type_condition_chain.next()?;
                match first {
                    TypeCondition::Interface(id) => FlatTypeCondition::Objects(sorted_intersection(
                        &schema[union_id].possible_types,
                        &schema[id].possible_types,
                    )),
                    TypeCondition::Object(id) => {
                        if schema[union_id].possible_types.contains(&id) {
                            FlatTypeCondition::Objects(Box::new([id]))
                        } else {
                            FlatTypeCondition::Objects(Box::new([]))
                        }
                    }
                    TypeCondition::Union(id) => FlatTypeCondition::Objects(sorted_intersection(
                        &schema[union_id].possible_types,
                        &schema[id].possible_types,
                    )),
                }
            }
        };

        for type_condition in type_condition_chain {
            candidate = match type_condition {
                TypeCondition::Interface(interface_id) => match candidate {
                    FlatTypeCondition::Interface(id) => {
                        if schema[interface_id].interfaces.contains(&id) {
                            FlatTypeCondition::Interface(id)
                        } else {
                            FlatTypeCondition::Objects(sorted_intersection(
                                &schema[interface_id].possible_types,
                                &schema[id].possible_types,
                            ))
                        }
                    }
                    FlatTypeCondition::Objects(ids) => {
                        FlatTypeCondition::Objects(sorted_intersection(&ids, &schema[interface_id].possible_types))
                    }
                },
                TypeCondition::Object(object_id) => match candidate {
                    FlatTypeCondition::Interface(id) => {
                        if schema[object_id].interfaces.contains(&id) {
                            FlatTypeCondition::Objects(Box::new([object_id]))
                        } else {
                            FlatTypeCondition::Objects(Box::new([]))
                        }
                    }
                    FlatTypeCondition::Objects(ids) => {
                        FlatTypeCondition::Objects(sorted_intersection(&ids, &[object_id]))
                    }
                },
                TypeCondition::Union(union_id) => match candidate {
                    FlatTypeCondition::Interface(id) => FlatTypeCondition::Objects(sorted_intersection(
                        &schema[union_id].possible_types,
                        &schema[id].possible_types,
                    )),
                    FlatTypeCondition::Objects(ids) => {
                        FlatTypeCondition::Objects(sorted_intersection(&ids, &schema[union_id].possible_types))
                    }
                },
            };
        }

        Some(candidate)
    }
}

fn sorted_intersection(left: &[ObjectId], right: &[ObjectId]) -> Box<[ObjectId]> {
    let mut l = 0;
    let mut r = 0;
    let mut result = vec![];
    while l < left.len() && r < right.len() {
        match left[l].cmp(&right[r]) {
            Ordering::Less => l += 1,
            Ordering::Equal => {
                result.push(left[l]);
                l += 1;
                r += 1;
            }
            Ordering::Greater => r += 1,
        }
    }
    result.into_boxed_slice()
}
