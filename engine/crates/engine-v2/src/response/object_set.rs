use std::sync::Arc;

use id_newtypes::IdRange;
use schema::{EntityId, ObjectId, Schema};

use super::{ResponseObjectId, ResponsePath};

id_newtypes::NonZeroU16! {
    ResponseObjectSetId,
}

#[derive(Debug, Clone)]
pub struct ResponseObjectRef {
    pub id: ResponseObjectId,
    pub path: ResponsePath,
    pub definition_id: ObjectId,
}

pub(crate) type ResponseObjectSet = Vec<ResponseObjectRef>;

pub(crate) struct TrackedResponseObjectSets {
    pub(super) ids: IdRange<ResponseObjectSetId>,
    pub(super) sets: Vec<ResponseObjectSet>,
}

impl TrackedResponseObjectSets {
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

const SET_INDEX_OFFSET: usize = 24;
const OBJECT_INDEX_MASK: u32 = (1 << SET_INDEX_OFFSET) - 1;

#[derive(Default, Clone)]
pub(crate) struct FilteredResponseObjectSet {
    sets: Vec<Arc<ResponseObjectSet>>,
    // Upper 8 bits in the set index, the 24 lower is the object index.
    indices: Vec<u32>,
}

impl FilteredResponseObjectSet {
    pub(crate) fn with_response_objects(mut self, refs: Arc<ResponseObjectSet>) -> Self {
        self.sets.push(refs);
        let n = self.indices.len();
        let set_idx = self.sets.len() - 1;
        assert!(set_idx < 1 << 8, "Too many sets");
        for i in 0..self.sets[set_idx].len() {
            self.indices.push((set_idx << SET_INDEX_OFFSET) as u32 | i as u32);
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
        self.sets.push(refs);

        let n = self.indices.len();
        let set_idx = self.sets.len() - 1;
        assert!(set_idx < 1 << 8, "Too many sets");

        match entity_id {
            EntityId::Interface(id) => {
                let possible_types = &schema[id].possible_types;
                for (i, item) in self.sets[set_idx].iter().enumerate() {
                    if possible_types.binary_search(&item.definition_id).is_ok() {
                        self.indices.push((set_idx << SET_INDEX_OFFSET) as u32 | i as u32);
                    }
                }
            }
            EntityId::Object(id) => {
                for (i, item) in self.sets[set_idx].iter().enumerate() {
                    if item.definition_id == id {
                        self.indices.push((set_idx << SET_INDEX_OFFSET) as u32 | i as u32);
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
            .map(|index| &self.sets[(index >> SET_INDEX_OFFSET) as usize][(index & OBJECT_INDEX_MASK) as usize])
    }
}

impl std::ops::Index<usize> for FilteredResponseObjectSet {
    type Output = ResponseObjectRef;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Out of bounds")
    }
}

pub(crate) struct ResponseObjectIterator<'set> {
    parent: &'set FilteredResponseObjectSet,
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
