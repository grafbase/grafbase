use std::collections::HashMap;

use schema::ObjectId;

mod error;
mod read;
mod write;

pub use error::GraphqlError;
pub use read::{ReadSelection, ReadSelectionSet, ResponseObjectsView};
pub use write::WriteSelectionSet;

use crate::execution::{StrId, Strings};

const DENSE_BIT_FLAG: u32 = 1 << 31;
const DENSE_BIT_MASK: u32 = DENSE_BIT_FLAG - 1;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ResponseObjectId(u32);

pub struct Response {
    pub strings: Strings,
    // will be None if an error propagated up to the root.
    root: Option<ResponseObjectId>,
    sparse_objects: Vec<ResponseSparseObject>,
    dense_objects: Vec<ResponseDenseObject>,
    errors: Vec<GraphqlError>,
}

// Only supporting additions for the current graph. Deletion are... tricky
// It shouldn't be that difficult to know whether a remaining plan still needs a field after
// execution plan creation. But it's definitely not efficient currently. I think we can at
// least wait until we face actual problems. We're focused on OLTP workloads, so might never
// happen.
impl Response {
    pub fn new(strings: Strings) -> Self {
        let root = ResponseSparseObject {
            object_id: None,
            fields: HashMap::new(),
        };
        Self {
            strings,
            root: Some(ResponseObjectId(0)),
            dense_objects: vec![],
            sparse_objects: vec![root],
            errors: vec![],
        }
    }

    pub fn push_sparse_object(&mut self, object: ResponseSparseObject) -> ResponseObjectId {
        self.sparse_objects.push(object);
        ResponseObjectId((self.sparse_objects.len() - 1) as u32)
    }

    pub fn push_dense_object(&mut self, object: ResponseDenseObject) -> ResponseObjectId {
        self.dense_objects.push(object);
        let id = (self.dense_objects.len() - 1) as u32;
        ResponseObjectId(id | DENSE_BIT_FLAG)
    }

    pub fn get(&self, id: ResponseObjectId) -> ResponseObject<'_> {
        if id.0 & DENSE_BIT_FLAG == 0 {
            ResponseObject::Sparse(&self.sparse_objects[id.0 as usize])
        } else {
            ResponseObject::Dense(&self.dense_objects[(id.0 & DENSE_BIT_MASK) as usize])
        }
    }

    pub fn get_mut(&mut self, id: ResponseObjectId) -> ResponseMutObject<'_> {
        if id.0 & DENSE_BIT_FLAG == 0 {
            ResponseMutObject::Sparse(&mut self.sparse_objects[id.0 as usize])
        } else {
            ResponseMutObject::Dense(&mut self.dense_objects[(id.0 & DENSE_BIT_MASK) as usize])
        }
    }
}

#[repr(u8)]
pub enum ResponseObject<'a> {
    Sparse(&'a ResponseSparseObject),
    Dense(&'a ResponseDenseObject),
}

impl<'a> ResponseObject<'a> {
    fn object_id(&self) -> Option<ObjectId> {
        match self {
            Self::Sparse(obj) => obj.object_id,
            Self::Dense(_) => None,
        }
    }

    fn field(&self, position: usize, name: StrId) -> Option<&ResponseValue> {
        match self {
            Self::Sparse(obj) => obj.fields.get(&name),
            Self::Dense(obj) => Some(&obj.fields[position]),
        }
    }
}

#[repr(u8)]
pub enum ResponseMutObject<'a> {
    Sparse(&'a mut ResponseSparseObject),
    Dense(&'a mut ResponseDenseObject),
}

impl<'a> ResponseMutObject<'a> {
    fn insert(&mut self, position: usize, name: StrId, value: ResponseValue) {
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
pub struct ResponseSparseObject {
    // object_id will only be present if __typename was retrieved which always be the case
    // through proper planning when it's needed for unions/interfaces between plans.
    object_id: Option<ObjectId>,
    fields: HashMap<StrId, ResponseValue>,
}

#[derive(Debug)]
pub struct ResponseDenseObject {
    // fields placed by their position in the query.
    // This will only be used if there aren't any type conditions for now. We might want to have
    // something a bit smarter later, but that might be already too smart for my own good.
    fields: Vec<ResponseValue>,
}

#[derive(Debug)]
pub enum ResponseValue {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String), // We should probably intern enums.
    List(Vec<ResponseValue>),
    Object(ResponseObjectId),
}

impl ResponseValue {
    fn as_object(&self) -> Option<ResponseObjectId> {
        match self {
            Self::Object(id) => Some(*id),
            _ => None,
        }
    }
}
