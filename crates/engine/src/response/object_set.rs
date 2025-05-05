use std::sync::Arc;

use schema::{CompositeTypeId, ObjectDefinitionId, Schema};

use super::{ResponseObjectId, ResponseValueId};

/// A "fat" reference to a response object. We keep track of its path for further execution and its
/// definition id because we don't store it anywhere else as of today.
#[derive(Debug, Clone)]
pub(crate) struct ResponseObjectRef {
    pub id: ResponseObjectId,
    pub path: Vec<ResponseValueId>,
    pub definition_id: ObjectDefinitionId,
}

/// A ResponseObjectSet hols all the response object references for a given selection sets,
/// eventually with some filtering.
pub(crate) type ResponseObjectSet = Vec<ResponseObjectRef>;

const SET_INDEX_SHIFT: u32 = 24;
const MAX_SET_INDEX: usize = (1 << (u32::BITS - SET_INDEX_SHIFT)) as usize;
const OBJECT_INDEX_MASK: u32 = (1 << SET_INDEX_SHIFT) - 1;

/// An individual ResponseObjectSet may contain more objects than what a ResponseModifier or Plan
/// requires. ResponseObjectSet are built by accumulating all the response object references for a
/// given selection set (roughly), but a response modifier/plan may only apply on some object (type
/// condition). Moreover a ResponseModifier may be applied at different selection sets (containing
/// the same field/object).
///
/// So the `ParentObjects` abstracts all of this by providing an Iterator over all the
/// relevant the references for a given ResponseModifier/Plan. We eagerly compute the indices of
/// all the relevant references mostly out of simplicity. Typically for a federation entity
/// response, any errors will include the index in the response path, so we need an easy way to
/// find its respective path on our side.
#[derive(Default, Clone)]
pub(crate) struct ParentObjects {
    sets: Vec<Arc<ResponseObjectSet>>,
    // Upper 8 bits in the set index, the 24 lower is the object index.
    indices: Vec<u32>,
}

impl ParentObjects {
    pub fn with_response_objects(mut self, refs: Arc<ResponseObjectSet>) -> Self {
        let n = self.indices.len();
        self.indices.reserve_exact(refs.len());
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

    pub fn with_filtered_response_objects(
        mut self,
        schema: &Schema,
        ty_id: CompositeTypeId,
        refs: Arc<ResponseObjectSet>,
    ) -> Self {
        let n = self.indices.len();
        self.indices.reserve_exact(refs.len());

        let set_idx = self.sets.len();
        assert!(set_idx < MAX_SET_INDEX, "Too many sets");

        match ty_id {
            CompositeTypeId::Union(id) => {
                let possible_types = &schema[id].possible_type_ids;
                for (i, item) in refs.iter().enumerate() {
                    if possible_types.binary_search(&item.definition_id).is_ok() {
                        self.indices.push((set_idx << SET_INDEX_SHIFT) as u32 | i as u32);
                    }
                }
            }
            CompositeTypeId::Interface(id) => {
                let possible_types = &schema[id].possible_type_ids;
                for (i, item) in refs.iter().enumerate() {
                    if possible_types.binary_search(&item.definition_id).is_ok() {
                        self.indices.push((set_idx << SET_INDEX_SHIFT) as u32 | i as u32);
                    }
                }
            }
            CompositeTypeId::Object(id) => {
                for (i, item) in refs.iter().enumerate() {
                    if item.definition_id == id {
                        self.indices.push((set_idx << SET_INDEX_SHIFT) as u32 | i as u32);
                    }
                }
            }
        }
        self.sets.push(refs);
        assert!(
            self.indices.len() - n < (OBJECT_INDEX_MASK as usize),
            "Too many response objects"
        );

        self
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &ResponseObjectRef> {
        self.into_iter()
    }

    pub fn iter_with_id(&self) -> impl Iterator<Item = (ParentObjectId, &ResponseObjectRef)> {
        self.indices.iter().enumerate().map(move |(id, index)| {
            let set_idex = (index >> SET_INDEX_SHIFT) as usize;
            let object_index = (index & OBJECT_INDEX_MASK) as usize;
            (ParentObjectId(id as u32), &self.sets[set_idex][object_index])
        })
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn get(&self, i: usize) -> Option<&ResponseObjectRef> {
        self.indices
            .get(i)
            .map(|index| &self.sets[(index >> SET_INDEX_SHIFT) as usize][(index & OBJECT_INDEX_MASK) as usize])
    }
}

#[derive(Clone, Copy, PartialEq, Eq, id_derives::Id)]
pub(crate) struct ParentObjectId(u32);

impl std::ops::Index<ParentObjectId> for ParentObjects {
    type Output = ResponseObjectRef;
    fn index(&self, index: ParentObjectId) -> &Self::Output {
        self.get(usize::from(index)).expect("Out of bounds")
    }
}

impl std::ops::Index<usize> for ParentObjects {
    type Output = ResponseObjectRef;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Out of bounds")
    }
}

impl<'a> IntoIterator for &'a ParentObjects {
    type Item = &'a ResponseObjectRef;
    type IntoIter = ParentObjectIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        ParentObjectIter {
            parent_objects: self,
            indices_iter: self.indices.iter(),
        }
    }
}

pub(crate) struct ParentObjectIter<'a> {
    parent_objects: &'a ParentObjects,
    indices_iter: std::slice::Iter<'a, u32>,
}

impl ExactSizeIterator for ParentObjectIter<'_> {
    fn len(&self) -> usize {
        self.indices_iter.len()
    }
}

impl<'a> Iterator for ParentObjectIter<'a> {
    type Item = &'a ResponseObjectRef;
    fn next(&mut self) -> Option<Self::Item> {
        self.indices_iter.next().map(|index| {
            let set_idex = (index >> SET_INDEX_SHIFT) as usize;
            let object_index = (index & OBJECT_INDEX_MASK) as usize;
            &self.parent_objects.sets[set_idex][object_index]
        })
    }
}
