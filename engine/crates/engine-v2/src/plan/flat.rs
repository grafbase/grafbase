use std::{
    borrow::Borrow,
    collections::{HashSet, VecDeque},
};

use itertools::Itertools;
use schema::{Definition, EntityId, Schema};

use crate::operation::{FieldId, Operation, Selection, SelectionSetId, SelectionSetType, TypeCondition};

pub fn flatten_selection_sets(
    schema: &Schema,
    operation: &Operation,
    root_selection_set_ids: Vec<SelectionSetId>,
) -> FlatSelectionSet {
    let ty = {
        let selection_set_types = root_selection_set_ids
            .iter()
            .map(|id| operation[*id].ty)
            .collect::<HashSet<SelectionSetType>>();
        assert_eq!(
            selection_set_types.len(),
            1,
            "{}",
            selection_set_types
                .into_iter()
                .map(|ty| schema.walk(Definition::from(ty)).name())
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
            operation[selection_set_id]
                .items
                .iter()
                .map(move |selection| (vec![selection_set_id], selection))
        },
    ));
    while let Some((mut selection_set_path, selection)) = selections.pop_front() {
        match selection {
            &Selection::Field(field_id) => {
                flat_selection_set.fields.push(FlatField {
                    entity_id: operation[field_id]
                        .definition_id()
                        .map(|id| schema.walk(id).parent_entity().id())
                        .or_else(|| {
                            // Without a definition the field is a __typename, so we just use the
                            // last type condition if there was any.
                            if selection_set_path.len() == 1 {
                                return None;
                            }
                            selection_set_path.iter().rev().find_map(|id| match operation[*id].ty {
                                SelectionSetType::Object(id) => Some(EntityId::Object(id)),
                                SelectionSetType::Interface(id) => Some(EntityId::Interface(id)),
                                SelectionSetType::Union(_) => None,
                            })
                        }),
                    selection_set_path,
                    id: field_id,
                });
            }
            Selection::FragmentSpread(spread_id) => {
                let spread = &operation[*spread_id];
                selection_set_path.push(spread.selection_set_id);
                selections.extend(
                    operation[spread.selection_set_id]
                        .items
                        .iter()
                        .map(|selection| (selection_set_path.clone(), selection)),
                );
            }
            Selection::InlineFragment(inline_fragment_id) => {
                let inline_fragment = &operation[*inline_fragment_id];
                selection_set_path.push(inline_fragment.selection_set_id);
                selections.extend(
                    operation[inline_fragment.selection_set_id]
                        .items
                        .iter()
                        .map(|selection| (selection_set_path.clone(), selection)),
                );
            }
        }
    }

    flat_selection_set
}

#[derive(Debug, Clone)]
pub(crate) struct FlatSelectionSet {
    pub ty: SelectionSetType,
    pub root_selection_set_ids: Vec<SelectionSetId>,
    pub fields: Vec<FlatField>,
}

impl FlatSelectionSet {
    pub fn empty(ty: SelectionSetType) -> Self {
        Self {
            ty,
            root_selection_set_ids: Vec::new(),
            fields: Vec::new(),
        }
    }

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
    // There is always at least one element.
    pub selection_set_path: Vec<SelectionSetId>,
    pub entity_id: Option<EntityId>,
    pub id: FieldId,
}

impl Borrow<FieldId> for FlatField {
    fn borrow(&self) -> &FieldId {
        &self.id
    }
}
impl Borrow<FieldId> for &FlatField {
    fn borrow(&self) -> &FieldId {
        &self.id
    }
}

impl FlatField {
    pub fn parent_selection_set_id(&self) -> SelectionSetId {
        self.selection_set_path.last().copied().unwrap()
    }
}

impl From<EntityId> for SelectionSetType {
    fn from(value: EntityId) -> Self {
        match value {
            EntityId::Interface(id) => SelectionSetType::Interface(id),
            EntityId::Object(id) => SelectionSetType::Object(id),
        }
    }
}

impl From<EntityId> for TypeCondition {
    fn from(value: EntityId) -> Self {
        match value {
            EntityId::Interface(id) => TypeCondition::Interface(id),
            EntityId::Object(id) => TypeCondition::Object(id),
        }
    }
}
