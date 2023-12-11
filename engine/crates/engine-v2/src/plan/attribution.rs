use std::collections::{HashMap, HashSet};

use schema::{FieldId, ResolverWalker};

use super::ExpectedType;
use crate::{
    request::{BoundFieldId, BoundSelectionSetId, FlatTypeCondition, SelectionSetType},
    response::{ResponseEdge, ResponseKeys},
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

#[derive(Debug)]
pub struct ExtraField {
    pub edge: ResponseEdge,
    pub type_condition: Option<FlatTypeCondition>,
    pub field_id: FieldId,
    pub expected_key: String,
    pub ty: ExpectedType<ExtraSelectionSetId>,
}

#[derive(Debug)]
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
    pub fn get(&self) -> &'a <Attribution as std::ops::Index<Id>>::Output {
        &self.attribution[self.id]
    }
}

pub type ExtraSelectionSetWalker<'a> = AttributionWalker<'a, ExtraSelectionSetId>;
pub type ExtraFieldWalker<'a> = AttributionWalker<'a, ExtraFieldId>;

impl<'a> ExtraSelectionSetWalker<'a> {
    pub fn ty(&self) -> SelectionSetType {
        self.get().ty
    }

    pub fn fields(&self) -> impl Iterator<Item = ExtraFieldWalker<'a>> + 'a {
        let walker = self.walk(());
        self.get().fields.iter().map(move |id| walker.walk(*id))
    }
}

impl<'a> ExtraFieldWalker<'a> {
    pub fn selection_set(&self) -> Option<ExtraSelectionSetWalker<'a>> {
        match self.get().ty {
            ExpectedType::Scalar(_) => None,
            ExpectedType::SelectionSet(id) => Some(self.walk(id)),
        }
    }

    pub fn expected_key(&self) -> &'a str {
        &self.get().expected_key
    }
}

impl<'a> std::ops::Deref for ExtraFieldWalker<'a> {
    type Target = ExtraField;

    fn deref(&self) -> &'a Self::Target {
        self.get()
    }
}

#[derive(Debug)]
pub(super) struct AttributionBuilder<'a> {
    response_keys: &'a ResponseKeys,
    resolver: ResolverWalker<'a>,
    extra_fields: Vec<ExtraField>,
    extra_selection_sets: Vec<ExtraSelectionSetBuilder>,
    extra_field_names: HashMap<FieldId, String>,
    pub attributed_selection_sets: HashSet<BoundSelectionSetId>,
    pub attributed_fields: Vec<BoundFieldId>,
    pub extras: HashMap<BoundSelectionSetId, ExtraSelectionSetId>,
}

impl<'a> std::ops::Index<ExtraFieldId> for AttributionBuilder<'a> {
    type Output = ExtraField;

    fn index(&self, index: ExtraFieldId) -> &Self::Output {
        &self.extra_fields[usize::from(index)]
    }
}

impl<'a> std::ops::Index<ExtraSelectionSetId> for AttributionBuilder<'a> {
    type Output = ExtraSelectionSetBuilder;

    fn index(&self, index: ExtraSelectionSetId) -> &Self::Output {
        &self.extra_selection_sets[usize::from(index)]
    }
}

impl<'a> std::ops::IndexMut<ExtraSelectionSetId> for AttributionBuilder<'a> {
    fn index_mut(&mut self, index: ExtraSelectionSetId) -> &mut Self::Output {
        &mut self.extra_selection_sets[usize::from(index)]
    }
}

impl<'a> AttributionBuilder<'a> {
    pub fn new(response_keys: &'a ResponseKeys, resolver: ResolverWalker<'a>) -> Self {
        Self {
            response_keys,
            resolver,
            attributed_selection_sets: HashSet::new(),
            attributed_fields: Vec::new(),
            extra_fields: Vec::new(),
            extra_selection_sets: Vec::new(),
            extras: HashMap::new(),
            extra_field_names: HashMap::new(),
        }
    }

    pub fn extra_fields(&self, id: BoundSelectionSetId) -> Option<impl Iterator<Item = &ExtraField> + '_> {
        self.extras.get(&id).map(|id| {
            self.extra_selection_sets[usize::from(*id)]
                .fields
                .values()
                .map(|id| &self.extra_fields[usize::from(*id)])
        })
    }

    pub fn extra_field_ids(&self, id: BoundSelectionSetId) -> Option<impl Iterator<Item = ExtraFieldId> + '_> {
        self.extras
            .get(&id)
            .map(|id| self.extra_selection_sets[usize::from(*id)].fields.values().copied())
    }

    pub fn extra_selection_set_for(&mut self, id: BoundSelectionSetId, ty: SelectionSetType) -> ExtraSelectionSetId {
        *self.extras.entry(id).or_insert_with(|| {
            let id = ExtraSelectionSetId::from(self.extra_selection_sets.len());
            self.extra_selection_sets.push(ExtraSelectionSetBuilder {
                ty,
                fields: HashMap::new(),
            });
            id
        })
    }

    pub fn get_or_insert_extra_field_with(
        &mut self,
        extra_selection_set_id: ExtraSelectionSetId,
        type_condition: Option<&FlatTypeCondition>,
        field_id: FieldId,
    ) -> &ExtraField {
        // Clippy doesn't see the ownership problem, we need insert a extra selection_set during the
        // creation of the extra field. So we can't borrow the extra_selection_sets during the
        // extra field creation.
        #[allow(clippy::map_entry)]
        let extra_field_id = if !self[extra_selection_set_id].fields.contains_key(&field_id) {
            let extra_field_id = ExtraFieldId::from(self.extra_fields.len());
            let field = self.resolver.walk(field_id);
            self.extra_fields.push(ExtraField {
                edge: field_id.into(),
                field_id,
                type_condition: type_condition.cloned(),
                expected_key: {
                    if self.resolver.supports_aliases() {
                        // When the resolver supports aliases, we must ensure that extra fields
                        // don't collide with existing response keys. And to avoid duplicates
                        // during field collection, we have a single unique name per field id.
                        self.extra_field_names
                            .entry(field_id)
                            .or_insert_with(|| {
                                let short_id = hex::encode(u32::from(field_id).to_be_bytes())
                                    .trim_start_matches('0')
                                    .to_uppercase();
                                let name = format!("_extra{}_{}", short_id, field.name());
                                // name is unique, but may collide with existing keys so
                                // iterating over candidates until we find a valid one.
                                // This is only a safeguard, it most likely won't ever run.
                                if self.response_keys.contains(&name) {
                                    let mut index = 0;
                                    loop {
                                        let candidate = format!("{name}_{index}");
                                        if !self.response_keys.contains(&candidate) {
                                            break candidate;
                                        }
                                        index += 1;
                                    }
                                } else {
                                    name
                                }
                            })
                            .to_string()
                    } else {
                        field.name().to_string()
                    }
                },
                ty: field
                    .ty()
                    .inner()
                    .data_type()
                    .map(ExpectedType::Scalar)
                    .unwrap_or_else(|| {
                        let id = ExtraSelectionSetId::from(self.extra_selection_sets.len());
                        self.extra_selection_sets.push(ExtraSelectionSetBuilder {
                            ty: SelectionSetType::maybe_from(field.ty().inner().id()).unwrap(),
                            fields: HashMap::new(),
                        });
                        ExpectedType::SelectionSet(id)
                    }),
            });
            self[extra_selection_set_id].fields.insert(field_id, extra_field_id);
            extra_field_id
        } else {
            self[extra_selection_set_id].fields[&field_id]
        };
        &self[extra_field_id]
    }

    pub fn build(self) -> Attribution {
        let mut attribution = Attribution {
            attributed_selection_sets: self.attributed_selection_sets.into_iter().collect(),
            attributed_fields: self.attributed_fields,
            extra_fields: self.extra_fields,
            extra_selection_sets: self
                .extra_selection_sets
                .into_iter()
                .map(ExtraSelectionSetBuilder::build)
                .collect(),
            extras: self.extras,
        };
        attribution.attributed_fields.sort_unstable();
        attribution.attributed_selection_sets.sort_unstable();
        attribution
    }
}

#[derive(Clone, Debug)]
pub struct ExtraSelectionSetBuilder {
    pub ty: SelectionSetType,
    pub fields: HashMap<FieldId, ExtraFieldId>,
}

impl ExtraSelectionSetBuilder {
    pub fn build(self) -> ExtraSelectionSet {
        ExtraSelectionSet {
            ty: self.ty,
            fields: self.fields.into_values().collect(),
        }
    }
}
