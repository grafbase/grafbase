use std::sync::Arc;

use id_newtypes::IdRange;
use schema::{EntityId, ObjectId, Schema};

use super::{ResponseObjectId, ResponsePath};

id_newtypes::NonZeroU16! {
    ResponseObjectSetId,
}

/// A "fat" reference to a response object. We keep track of its path for further execution and its
/// definition id because we don't store it anywhere else as of today.
#[derive(Debug, Clone)]
pub struct ResponseObjectRef {
    pub id: ResponseObjectId,
    pub path: ResponsePath,
    pub definition_id: ObjectId,
}

/// A ResponseObjectSet hols all the response object references for a given selection sets,
/// eventually with some filtering.
pub(crate) type ResponseObjectSet = Vec<ResponseObjectRef>;

/// A Plan can be summarized to adding fields to an existing response object. Root plan obviously update the
/// root object (Query, etc..). All other plan root response objects are produced by a parent
/// plan. So a parent plan will keep track of all the response objects that will be used by later
/// plans or response modifiers. `OutputResponseObjectSets` contains all of those and is created
/// after plan execution.
pub(crate) struct OutputResponseObjectSets {
    pub(super) ids: IdRange<ResponseObjectSetId>,
    pub(super) sets: Vec<ResponseObjectSet>,
}

impl OutputResponseObjectSets {
    #[allow(unused)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (ResponseObjectSetId, &mut ResponseObjectSet)> + '_ {
        self.ids
            .into_iter()
            .zip(self.sets.iter_mut())
            .filter(|(_, set)| !set.is_empty())
    }

    pub fn into_iter(self) -> impl Iterator<Item = (ResponseObjectSetId, ResponseObjectSet)> {
        self.ids.into_iter().zip(self.sets).filter(|(_, set)| !set.is_empty())
    }
}

const SET_INDEX_SHIFT: u32 = 24;
const MAX_SET_INDEX: usize = (1 << (u32::BITS - SET_INDEX_SHIFT)) as usize;
const OBJECT_INDEX_MASK: u32 = (1 << SET_INDEX_SHIFT) - 1;

/// An individual ResponseObjectSet may contain more objects than what a ResponseModifier or Plan
/// requires. ResponseObjectSet are built by accumulating all the response object references for a
/// given selection set (roughly), but a response modifier/plan may only apply on some object (type
/// condition). Moreover a ResponseModifier may be applied at different selection sets (containing
/// the same field/object).
///
/// So the `InputResponseObjectSet` abstracts all of this by providing an Iterator over all the
/// relevant the references for a given ResponseModifier/Plan. We eagerly compute the indices of
/// all the relevant references mostly out of simplicity. Typically for a federation entity
/// response, any errors will include the index in the response path, so we need an easy way to
/// find its respective path on our side.
#[derive(Default, Clone)]
pub(crate) struct InputdResponseObjectSet {
    sets: Vec<Arc<ResponseObjectSet>>,
    // Upper 8 bits in the set index, the 24 lower is the object index.
    indices: Vec<u32>,
}

impl InputdResponseObjectSet {
    pub(crate) fn with_response_objects(mut self, refs: Arc<ResponseObjectSet>) -> Self {
        let n = self.indices.len();
        self.indices.reserve(refs.len());
        self.sets.push(refs);

        let set_idx = self.sets.len() - 1;
        assert!(set_idx < MAX_SET_INDEX, "Too many sets");
        for i in 0..self.sets[set_idx].len() {
            self.indices.push((set_idx << SET_INDEX_SHIFT) as u32 | i as u32);
        }
        assert!(
            self.indices.len() - n < (OBJECT_INDEX_MASK as usize),
            "Too many response objects"
        );
        self
    }

    pub(crate) fn with_filtered_response_objects(
        mut self,
        schema: &Schema,
        entity_id: EntityId,
        refs: Arc<ResponseObjectSet>,
    ) -> Self {
        let n = self.indices.len();
        self.indices.reserve(refs.len());
        self.sets.push(refs);

        let set_idx = self.sets.len() - 1;
        assert!(set_idx < MAX_SET_INDEX, "Too many sets");

        match entity_id {
            EntityId::Interface(id) => {
                let possible_types = &schema[id].possible_types;
                for (i, item) in self.sets[set_idx].iter().enumerate() {
                    if possible_types.binary_search(&item.definition_id).is_ok() {
                        self.indices.push((set_idx << SET_INDEX_SHIFT) as u32 | i as u32);
                    }
                }
            }
            EntityId::Object(id) => {
                for (i, item) in self.sets[set_idx].iter().enumerate() {
                    if item.definition_id == id {
                        self.indices.push((set_idx << SET_INDEX_SHIFT) as u32 | i as u32);
                    }
                }
            }
        }
        assert!(
            self.indices.len() - n < (OBJECT_INDEX_MASK as usize),
            "Too many response objects"
        );

        self
    }

    pub(crate) fn iter(&self) -> ResponseObjectIterator<'_> {
        ResponseObjectIterator { parent: self, idx: 0 }
    }

    pub(crate) fn len(&self) -> usize {
        self.indices.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub(crate) fn get(&self, i: usize) -> Option<&ResponseObjectRef> {
        self.indices
            .get(i)
            .map(|index| &self.sets[(index >> SET_INDEX_SHIFT) as usize][(index & OBJECT_INDEX_MASK) as usize])
    }
}

impl std::ops::Index<usize> for InputdResponseObjectSet {
    type Output = ResponseObjectRef;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Out of bounds")
    }
}

pub(crate) struct ResponseObjectIterator<'set> {
    parent: &'set InputdResponseObjectSet,
    idx: usize,
}

impl<'set> Iterator for ResponseObjectIterator<'set> {
    type Item = &'set ResponseObjectRef;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.parent.get(self.idx)?;
        self.idx += 1;
        Some(item)
    }
}
