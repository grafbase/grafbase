// mod de;
mod deserialize;
mod ids;
mod manual;
mod scalar;
mod writer;

use std::{collections::BTreeMap, sync::Arc};

pub use ids::*;
pub use manual::*;
use schema::Schema;
pub use writer::*;

use super::{
    BoundResponseKey, GraphqlError, InitialResponse, ResponseData, ResponseKeys, ResponseObject, ResponsePath,
    ResponseValue,
};
// use de::AnyFieldsSeed;
// use serde::de::DeserializeSeed;
use crate::{request::Operation, Response};

#[derive(Default)]
pub struct ResponseDataPart {
    objects: Vec<ResponseObject>,
    lists: Vec<ResponseValue>,
}

impl ResponseDataPart {
    fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

pub struct ResponseBuilder {
    pub(super) keys: ResponseKeys,
    // will be None if an error propagated up to the root.
    pub(super) root: Option<ResponseObjectId>,
    parts: Vec<ResponseDataPart>,
    errors: Vec<GraphqlError>,
}

// Only supporting additions for the current graph. Deletion are... tricky
// It shouldn't be that difficult to know whether a remaining plan still needs a field after
// execution plan creation. But it's definitely not efficient currently. I think we can at
// least wait until we face actual problems. We're focused on OLTP workloads, so might never
// happen.
impl ResponseBuilder {
    pub fn new(schema: &Schema, operation: &Operation) -> Self {
        let mut builder = ResponsePartBuilder::new(ResponseDataPartId::from(0));
        let root_id = {
            let typename = schema[operation.root_object_id].name;
            builder.push_object(ResponseObject {
                object_id: operation.root_object_id,
                fields: operation
                    .walk_selection_set(schema.default_walker(), operation.root_selection_set_id)
                    .fields()
                    .filter(|field| field.definition().is_typename_meta_field())
                    .map(|field| {
                        (
                            field.bound_response_key(),
                            ResponseValue::StringId {
                                id: typename,
                                nullable: false,
                            },
                        )
                    })
                    .collect(),
            })
        };
        Self {
            keys: operation.response_keys.clone(),
            root: Some(root_id),
            parts: vec![builder.part],
            errors: vec![],
        }
    }

    pub fn new_part(&mut self) -> ResponsePartBuilder {
        let id = ResponseDataPartId::from(self.parts.len());
        // reserving the spot until the actual data is written. It's safe as no one can reference
        // any data in this part before it's added. And a part can only be overwritten if it's
        // empty.
        self.parts.push(ResponseDataPart::default());
        ResponsePartBuilder::new(id)
    }

    pub fn ingest_part(&mut self, builder: ResponsePartBuilder) {
        let reservation = &mut self.parts[usize::from(builder.id)];
        assert!(reservation.is_empty(), "Part already has data");
        *reservation = builder.part;
        self.errors.extend(builder.errors);
        for update in builder.updates {
            self[update.id].fields.extend(update.fields);
        }
        for error in builder.errors_to_propagate {
            self.propagate_error(error);
        }
    }

    pub fn push_error(&mut self, error: impl Into<GraphqlError>) {
        self.errors.push(error.into());
    }

    pub fn build(self, schema: Arc<Schema>) -> Response {
        Response::Initial(InitialResponse {
            data: ResponseData {
                schema,
                keys: self.keys,
                root: self.root,
                parts: self.parts,
            },
            errors: self.errors,
        })
    }

    // The path corresponds to place where a plan failed but couldn't go propagate higher as data
    // was in a different part (provided by a parent plan).
    // To correctly propagate error we're finding the last nullable element in the path and make it
    // nullable. If there's nothing, then root will be null.
    fn propagate_error(&mut self, path: ResponsePath) {
        let Some(root) = self.root else {
            return;
        };
        let mut last_nullable: Option<ResponseValueId> = None;
        let mut previous: Result<ResponseObjectId, ResponseListId> = Ok(root);
        for segment in path.iter() {
            let (unique_id, value) = match (previous, segment.try_into_bound_response_key()) {
                (Ok(object_id), Ok(key)) => {
                    let unique_id = ResponseValueId::ObjectField { object_id, key };
                    let value = self[object_id].fields.get(&key);
                    (unique_id, value)
                }
                (Err(list_id), Err(index)) => {
                    let unique_id = ResponseValueId::ListItem { list_id, index };
                    let value = self[list_id].get(index);
                    (unique_id, value)
                }
                _ => return,
            };
            let Some(value) = value else {
                // Shouldn't happen but equivalent to null
                return;
            };
            if value.is_null() {
                return;
            }
            match value {
                ResponseValue::Object { id, nullable } => {
                    if *nullable {
                        last_nullable = Some(unique_id);
                    }
                    previous = Ok(*id);
                }
                ResponseValue::List { id, nullable } => {
                    if *nullable {
                        last_nullable = Some(unique_id);
                    }
                    previous = Err(*id);
                }
                _ => break,
            }
        }
        if let Some(last_nullable) = last_nullable {
            match last_nullable {
                ResponseValueId::ObjectField { object_id, key } => {
                    self[object_id].fields.insert(key, ResponseValue::Null);
                }
                ResponseValueId::ListItem { list_id, index } => {
                    self[list_id][index] = ResponseValue::Null;
                }
            }
        } else {
            self.root = None;
        }
    }
}

pub enum ResponseValueId {
    ObjectField {
        object_id: ResponseObjectId,
        key: BoundResponseKey,
    },
    ListItem {
        list_id: ResponseListId,
        index: usize,
    },
}

pub struct ResponsePartBuilder {
    id: ResponseDataPartId,
    part: ResponseDataPart,
    errors: Vec<GraphqlError>,
    updates: Vec<ResponseObjectUpdate>,
    errors_to_propagate: Vec<ResponsePath>,
}

impl ResponsePartBuilder {
    pub fn new(id: ResponseDataPartId) -> ResponsePartBuilder {
        ResponsePartBuilder {
            id,
            part: ResponseDataPart::default(),
            errors: Vec::new(),
            updates: Vec::new(),
            errors_to_propagate: Vec::new(),
        }
    }

    pub fn push_update(&mut self, update: ResponseObjectUpdate) {
        self.updates.push(update);
    }

    pub fn push_error(&mut self, error: impl Into<GraphqlError>) {
        self.errors.push(error.into());
    }

    pub fn push_error_to_propagate(&mut self, path: ResponsePath) {
        self.errors_to_propagate.push(path);
    }
}

pub struct ResponseObjectUpdate {
    pub id: ResponseObjectId,
    pub fields: BTreeMap<BoundResponseKey, ResponseValue>,
}
