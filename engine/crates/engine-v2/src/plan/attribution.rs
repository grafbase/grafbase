use std::collections::{HashMap, HashSet};

use schema::FieldId;

use super::{
    planner::{ExtraBoundaryField, ExtraBoundarySelectionSet},
    ExpectedType,
};
use crate::{
    request::{BoundFieldId, BoundSelectionSetId, FlatTypeCondition, SelectionSetType},
    response::ResponseEdge,
};

mod ids {
    use super::*;

    crate::utils::id_newtypes! {
        Attribution.extra_fields[ExtraFieldId] => ExtraField unless "Too many extra fields",
        Attribution.extra_selection_sets[ExtraSelectionSetId] => ExtraSelectionSet unless "Too many extra selection sets",
    }
}

pub use ids::*;

/// Keeps track of all fields associated to a certain plan. All selection_sets that have at least
/// one field, eventually nested inside a fragment, are also tracked to ensure we the plan doesn't
/// see any empty selection sets.
#[derive(Debug)]
pub struct Attribution {
    attributed_selection_sets: Vec<BoundSelectionSetId>,
    attributed_fields: Vec<BoundFieldId>,
    extras: HashMap<BoundSelectionSetId, ExtraSelectionSetId>,
    extra_fields: Vec<ExtraField>,
    extra_selection_sets: Vec<ExtraSelectionSet>,
}

impl Attribution {
    pub fn walk<Id>(&self, id: Id) -> AttributionWalker<'_, Id> {
        AttributionWalker { attribution: self, id }
    }

    pub fn field(&self, id: BoundFieldId) -> bool {
        self.attributed_fields.binary_search(&id).is_ok()
    }

    pub fn selection_set(&self, id: BoundSelectionSetId) -> bool {
        self.attributed_selection_sets.binary_search(&id).is_ok()
    }

    pub fn extras_for(&self, id: BoundSelectionSetId) -> Option<ExtraSelectionSetWalker<'_>> {
        self.extras.get(&id).map(|id| self.walk(*id))
    }
}

#[derive(Debug, Clone)]
pub struct ExtraField<SelectionSet = ExtraSelectionSetId> {
    pub edge: ResponseEdge,
    pub type_condition: Option<FlatTypeCondition>,
    pub field_id: FieldId,
    pub expected_key: String,
    pub ty: ExpectedType<SelectionSet>,
}

#[derive(Debug, Clone)]
pub struct ExtraSelectionSet {
    pub ty: SelectionSetType,
    pub fields: Vec<ExtraFieldId>,
}

#[derive(Clone, Copy)]
pub struct AttributionWalker<'a, Id> {
    attribution: &'a Attribution,
    id: Id,
}

impl<'a, Id> AttributionWalker<'a, Id> {
    fn walk<I2>(&self, id: I2) -> AttributionWalker<'a, I2> {
        AttributionWalker {
            attribution: self.attribution,
            id,
        }
    }
}

impl<'a, Id: Copy> AttributionWalker<'a, Id> {
    pub fn id(&self) -> Id {
        self.id
    }
}

impl<'a, Id: Copy> AttributionWalker<'a, Id>
where
    Attribution: std::ops::Index<Id>,
{
    pub fn as_ref(&self) -> &'a <Attribution as std::ops::Index<Id>>::Output {
        &self.attribution[self.id]
    }
}

pub type ExtraSelectionSetWalker<'a> = AttributionWalker<'a, ExtraSelectionSetId>;
pub type ExtraFieldWalker<'a> = AttributionWalker<'a, ExtraFieldId>;

impl<'a> ExtraSelectionSetWalker<'a> {
    pub fn ty(&self) -> SelectionSetType {
        self.as_ref().ty
    }

    pub fn fields(&self) -> impl Iterator<Item = ExtraFieldWalker<'a>> + 'a {
        let walker = self.walk(());
        self.as_ref().fields.iter().map(move |id| walker.walk(*id))
    }
}

impl<'a> ExtraFieldWalker<'a> {
    pub fn selection_set(&self) -> Option<ExtraSelectionSetWalker<'a>> {
        match self.as_ref().ty {
            ExpectedType::Scalar(_) => None,
            ExpectedType::SelectionSet(id) => Some(self.walk(id)),
        }
    }

    pub fn expected_key(&self) -> &'a str {
        &self.as_ref().expected_key
    }
}

impl<'a> std::ops::Deref for ExtraFieldWalker<'a> {
    type Target = ExtraField;

    fn deref(&self) -> &'a Self::Target {
        self.as_ref()
    }
}

#[derive(Debug, Default)]
pub(super) struct AttributionBuilder {
    extra_fields: Vec<ExtraField>,
    extra_selection_sets: Vec<ExtraSelectionSet>,
    pub attributed_selection_sets: HashSet<BoundSelectionSetId>,
    pub attributed_fields: Vec<BoundFieldId>,
    pub extras: HashMap<BoundSelectionSetId, ExtraSelectionSetId>,
}

impl std::ops::Index<ExtraFieldId> for AttributionBuilder {
    type Output = ExtraField;

    fn index(&self, index: ExtraFieldId) -> &Self::Output {
        &self.extra_fields[usize::from(index)]
    }
}

impl std::ops::Index<ExtraSelectionSetId> for AttributionBuilder {
    type Output = ExtraSelectionSet;

    fn index(&self, index: ExtraSelectionSetId) -> &Self::Output {
        &self.extra_selection_sets[usize::from(index)]
    }
}

impl std::ops::IndexMut<ExtraSelectionSetId> for AttributionBuilder {
    fn index_mut(&mut self, index: ExtraSelectionSetId) -> &mut Self::Output {
        &mut self.extra_selection_sets[usize::from(index)]
    }
}

impl AttributionBuilder {
    pub fn extra_fields(&self, id: BoundSelectionSetId) -> Option<impl Iterator<Item = &ExtraField> + '_> {
        self.extras.get(&id).map(|id| {
            self.extra_selection_sets[usize::from(*id)]
                .fields
                .iter()
                .map(|id| &self.extra_fields[usize::from(*id)])
        })
    }

    pub fn extra_field_ids(&self, id: BoundSelectionSetId) -> Option<impl Iterator<Item = ExtraFieldId> + '_> {
        self.extras
            .get(&id)
            .map(|id| self.extra_selection_sets[usize::from(*id)].fields.iter().copied())
    }

    pub fn add_extra_selection_sets(&mut self, extras: HashMap<BoundSelectionSetId, ExtraBoundarySelectionSet>) {
        for (id, extra) in extras {
            let extra_selection_set_id = self.insert_extra_selection_set(extra);
            self.attributed_selection_sets.insert(id);
            self.extras.insert(id, extra_selection_set_id);
        }
    }

    fn insert_extra_selection_set(&mut self, extra: ExtraBoundarySelectionSet) -> ExtraSelectionSetId {
        let selection_set = ExtraSelectionSet {
            ty: extra.ty,
            fields: extra
                .fields
                .into_values()
                .filter_map(
                    |ExtraBoundaryField {
                         extra_field,
                         read: used,
                     }| {
                        if used {
                            Some(extra_field)
                        } else {
                            None
                        }
                    },
                )
                .map(
                    |ExtraField {
                         edge,
                         type_condition,
                         field_id,
                         expected_key,
                         ty,
                     }| {
                        let field = ExtraField {
                            edge,
                            type_condition,
                            field_id,
                            expected_key,
                            ty: match ty {
                                ExpectedType::Scalar(scalar) => ExpectedType::Scalar(scalar),
                                ExpectedType::SelectionSet(extra) => {
                                    ExpectedType::SelectionSet(self.insert_extra_selection_set(extra))
                                }
                            },
                        };
                        let id = ExtraFieldId::from(self.extra_fields.len());
                        self.extra_fields.push(field);
                        id
                    },
                )
                .collect(),
        };
        let id = ExtraSelectionSetId::from(self.extra_selection_sets.len());
        self.extra_selection_sets.push(selection_set);
        id
    }

    pub fn build(self) -> Attribution {
        let mut attribution = Attribution {
            attributed_selection_sets: self.attributed_selection_sets.into_iter().collect(),
            attributed_fields: self.attributed_fields,
            extra_fields: self.extra_fields,
            extra_selection_sets: self.extra_selection_sets,
            extras: self.extras,
        };
        attribution.attributed_fields.sort_unstable();
        attribution.attributed_selection_sets.sort_unstable();
        attribution
    }
}
