use std::collections::HashMap;

use schema::ObjectId;

use super::ResponseValue;
use crate::execution::{StrId, Strings};

const COMPACT_BIT_FLAG: u32 = 1 << 31;
const COMPACT_BIT_MASK: u32 = COMPACT_BIT_FLAG - 1;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ResponseObjectId(u32);

#[derive(Debug)]
pub struct ResponseData {
    pub strings: Strings,
    // will be None if an error propagated up to the root.
    pub(super) root: Option<ResponseObjectId>,
    pub(super) objects: Vec<ResponseObject>,
    pub(super) compact_objects: Vec<CompactResponseObject>,
}

// Only supporting additions for the current graph. Deletion are... tricky
// It shouldn't be that difficult to know whether a remaining plan still needs a field after
// execution plan creation. But it's definitely not efficient currently. I think we can at
// least wait until we face actual problems. We're focused on OLTP workloads, so might never
// happen.
impl ResponseData {
    pub fn new(strings: Strings) -> Self {
        let root = ResponseObject {
            object_id: None,
            fields: HashMap::new(),
        };
        Self {
            strings,
            root: Some(ResponseObjectId(0)),
            compact_objects: vec![],
            objects: vec![root],
        }
    }

    pub fn push_object(&mut self, object: ResponseObject) -> ResponseObjectId {
        self.objects.push(object);
        ResponseObjectId((self.objects.len() - 1) as u32)
    }

    pub fn push_compact_object(&mut self, object: CompactResponseObject) -> ResponseObjectId {
        self.compact_objects.push(object);
        let id = (self.compact_objects.len() - 1) as u32;
        ResponseObjectId(id | COMPACT_BIT_FLAG)
    }

    pub fn get(&self, id: ResponseObjectId) -> AnyResponseObject<'_> {
        if id.0 & COMPACT_BIT_FLAG == 0 {
            AnyResponseObject::Sparse(&self.objects[id.0 as usize])
        } else {
            AnyResponseObject::Dense(&self.compact_objects[(id.0 & COMPACT_BIT_MASK) as usize])
        }
    }

    pub fn get_mut(&mut self, id: ResponseObjectId) -> AnyResponseMutObject<'_> {
        if id.0 & COMPACT_BIT_FLAG == 0 {
            AnyResponseMutObject::Sparse(&mut self.objects[id.0 as usize])
        } else {
            AnyResponseMutObject::Dense(&mut self.compact_objects[(id.0 & COMPACT_BIT_MASK) as usize])
        }
    }
}

pub enum AnyResponseObject<'a> {
    Sparse(&'a ResponseObject),
    Dense(&'a CompactResponseObject),
}

impl<'a> AnyResponseObject<'a> {
    pub fn object_id(&self) -> Option<ObjectId> {
        match self {
            Self::Sparse(obj) => obj.object_id,
            Self::Dense(_) => None,
        }
    }

    pub fn field(&self, position: usize, name: StrId) -> Option<&ResponseValue> {
        match self {
            Self::Sparse(obj) => obj.fields.get(&name),
            Self::Dense(obj) => Some(&obj.fields[position]),
        }
    }
}

pub enum AnyResponseMutObject<'a> {
    Sparse(&'a mut ResponseObject),
    Dense(&'a mut CompactResponseObject),
}

impl<'a> AnyResponseMutObject<'a> {
    pub fn insert(&mut self, position: usize, name: StrId, value: ResponseValue) {
        match self {
            Self::Sparse(obj) => {
                obj.fields.insert(name, value);
            }
            Self::Dense(obj) => {
                obj.fields[position] = value;
            }
        }
    }
}

#[derive(Debug)]
pub struct ResponseObject {
    // Nullable for now, it's presence will be enforced later to suppot `__typename`
    pub object_id: Option<ObjectId>,
    pub fields: HashMap<StrId, ResponseValue>,
}

#[derive(Debug)]
pub struct CompactResponseObject {
    // fields placed by their position in the query. Used when the number of fields doesn't
    // depending on the concrete object type. So no type condition or every fields under a single
    // one.
    pub fields: Vec<ResponseValue>,
}
