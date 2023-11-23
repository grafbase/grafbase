use std::collections::HashMap;

use schema::ObjectId;

use super::ResponseValue;
use crate::execution::{StrId, Strings};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ResponseObjectId(u32);

#[derive(Debug)]
pub struct ResponseData {
    pub strings: Strings,
    // will be None if an error propagated up to the root.
    pub(super) root: Option<ResponseObjectId>,
    pub(super) objects: Vec<ResponseObject>,
}

// Only supporting additions for the current graph. Deletion are... tricky
// It shouldn't be that difficult to know whether a remaining plan still needs a field after
// execution plan creation. But it's definitely not efficient currently. I think we can at
// least wait until we face actual problems. We're focused on OLTP workloads, so might never
// happen.
impl ResponseData {
    pub fn new(root_object_id: ObjectId, strings: Strings) -> Self {
        let root = ResponseObject {
            object_id: Some(root_object_id),
            fields: HashMap::new(),
        };
        Self {
            strings,
            root: Some(ResponseObjectId(0)),
            objects: vec![root],
        }
    }

    pub fn push_object(&mut self, object: ResponseObject) -> ResponseObjectId {
        self.objects.push(object);
        ResponseObjectId((self.objects.len() - 1) as u32)
    }

    pub fn get(&self, id: ResponseObjectId) -> &ResponseObject {
        &self.objects[id.0 as usize]
    }

    pub fn get_mut(&mut self, id: ResponseObjectId) -> &mut ResponseObject {
        &mut self.objects[id.0 as usize]
    }
}

#[derive(Debug)]
pub struct ResponseObject {
    // Nullable for now, it's presence will be enforced later to suppot `__typename`
    pub object_id: Option<ObjectId>,
    pub fields: HashMap<StrId, ResponseValue>,
}
