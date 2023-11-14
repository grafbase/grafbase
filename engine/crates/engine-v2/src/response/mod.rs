use std::sync::Arc;

use schema::{ObjectId, Schema};

mod de;
mod fields;
mod selection_set;
mod ser;
mod view;

pub use fields::{
    Argument, Pos, ResponseField, ResponseFieldId, ResponseFields, ResponseFieldsBuilder, ResponseStringId,
    TypeCondition,
};
pub use selection_set::{
    OperationSelection, OperationSelectionSet, ReadSelection, ReadSelectionSet, WriteSelection, WriteSelectionSet,
};
pub use view::ResponseObjectsView;

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResponsePath(Vec<ResponseFieldId>);

impl ResponsePath {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn child(&self, field: ResponseFieldId) -> Self {
        let mut child = Self(self.0.clone());
        child.0.push(field);
        child
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ResponseObjectId(usize);

pub struct Response {
    schema: Arc<Schema>,
    fields: ResponseFields,
    root: ResponseObjectId,
    objects: Vec<ResponseObject>,
}

// Only supporting additions for the current graph. Deletion are... tricky
// It shouldn't be that difficult to know whether a remaining plan still needs a field after
// execution plan creation. But it's definitely not efficient currently. I think we can at
// least wait until we face actual problems. We're focused on OLTP workloads, so might never
// happen.
impl Response {
    pub fn new(schema: Arc<Schema>, edges: ResponseFields) -> Self {
        let root = ResponseObject {
            object_id: None,
            fields: vec![],
        };
        Self {
            schema,
            fields: edges,
            root: ResponseObjectId(0),
            objects: vec![root],
        }
    }

    pub fn push_object(&mut self, object: ResponseObject) -> ResponseObjectId {
        self.objects.push(object);
        ResponseObjectId(self.objects.len() - 1)
    }

    /// Used to provide a view on the inputs objects of a plan.
    pub fn view<'a>(
        &'a mut self,
        path: &'a ResponsePath,
        selection_set: &'a ReadSelectionSet,
    ) -> Option<ResponseObjectsView<'a>> {
        let response_object_ids = self.find_matching_object_node_ids(path);
        if response_object_ids.is_empty() {
            None
        } else {
            Some(ResponseObjectsView {
                ids: response_object_ids,
                response: self,
                selection_set,
            })
        }
    }

    fn find_matching_object_node_ids(&self, path: &ResponsePath) -> Vec<ResponseObjectId> {
        let mut nodes = vec![self.root];

        for edge_id in &path.0 {
            let edge = &self.fields[*edge_id];
            if let Some(ref type_condition) = edge.type_condition {
                nodes = nodes
                    .into_iter()
                    .filter_map(|node_id| {
                        let node = &self[node_id];
                        let object_id = node
                            .object_id
                            .expect("Missing object_id on a node that is subject to a type condition.");
                        let type_matches = match type_condition {
                            TypeCondition::Interface(interface_id) => {
                                self.schema[object_id].implements_interfaces.contains(interface_id)
                            }
                            TypeCondition::Object(expected_object_id) => object_id == *expected_object_id,
                            TypeCondition::Union(union_id) => self.schema[*union_id].members.contains(&object_id),
                        };
                        if type_matches {
                            node.find_field(edge.name).and_then(|node| node.as_object())
                        } else {
                            None
                        }
                    })
                    .collect();
            } else {
                nodes = nodes
                    .into_iter()
                    .filter_map(|node_id| {
                        let node = &self[node_id];
                        node.find_field(edge.name).and_then(|node| node.as_object())
                    })
                    .collect();
            }
            if nodes.is_empty() {
                break;
            }
        }

        nodes
    }
}

impl std::ops::Index<ResponseFieldId> for Response {
    type Output = ResponseField;

    fn index(&self, index: ResponseFieldId) -> &Self::Output {
        &self.fields[index]
    }
}

impl std::ops::Index<ResponseObjectId> for Response {
    type Output = ResponseObject;

    fn index(&self, index: ResponseObjectId) -> &Self::Output {
        &self.objects[index.0]
    }
}

impl std::ops::IndexMut<ResponseObjectId> for Response {
    fn index_mut(&mut self, index: ResponseObjectId) -> &mut Self::Output {
        &mut self.objects[index.0]
    }
}

impl std::ops::Index<ResponseStringId> for Response {
    type Output = str;

    fn index(&self, index: ResponseStringId) -> &Self::Output {
        &self.fields[index]
    }
}

pub struct ResponseObject {
    // object_id will only be present if __typename was retrieved which always be the case
    // through proper planning when it's needed for unions/interfaces between plans.
    object_id: Option<ObjectId>,
    fields: Vec<(ResponseStringId, ResponseValue)>,
}

impl ResponseObject {
    pub fn insert_fields(&mut self, fields: impl IntoIterator<Item = (ResponseStringId, ResponseValue)>) {
        self.fields.extend(fields);
    }

    pub fn find_field(&self, target: ResponseStringId) -> Option<&ResponseValue> {
        self.fields
            .iter()
            .find_map(|(name, node)| if *name == target { Some(node) } else { None })
    }
}

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
