mod deserialize;
mod ids;
mod manual;
mod writer;

use std::{collections::BTreeMap, sync::Arc};

pub use ids::*;
use itertools::Either;
pub use manual::*;
use schema::Schema;
pub use writer::*;

use super::{
    ExecutionMetadata, GraphqlError, InitialResponse, ResponseBoundaryItem, ResponseData, ResponseEdge, ResponseKeys,
    ResponseObject, ResponsePath, ResponseValue, UnpackedResponseEdge,
};
use crate::{
    plan::{PlanBoundary, PlanBoundaryId},
    request::Operation,
    Response,
};

#[derive(Default)]
pub(crate) struct ResponseDataPart {
    objects: Vec<ResponseObject>,
    lists: Vec<ResponseValue>,
}

impl ResponseDataPart {
    fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

pub(crate) struct ResponseBuilder {
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
    pub fn new(operation: &Operation) -> Self {
        let mut builder = ExecutorOutput::new(ResponseDataPartId::from(0), vec![]);
        let root_id = builder.push_object(ResponseObject {
            object_id: operation.root_object_id,
            fields: BTreeMap::new(),
        });
        Self {
            root: Some(root_id),
            parts: vec![builder.data_part],
            errors: vec![],
        }
    }

    pub fn root_response_object_id(&self) -> Option<ResponseObjectId> {
        self.root
    }

    pub fn new_output(&mut self, boundaries: Vec<PlanBoundary>) -> ExecutorOutput {
        let id = ResponseDataPartId::from(self.parts.len());
        // reserving the spot until the actual data is written. It's safe as no one can reference
        // any data in this part before it's added. And a part can only be overwritten if it's
        // empty.
        self.parts.push(ResponseDataPart::default());
        ExecutorOutput::new(id, boundaries)
    }

    pub fn root_response_boundary(&self) -> Option<ResponseBoundaryItem> {
        Some(ResponseBoundaryItem {
            response_object_id: self.root?,
            response_path: ResponsePath::default(),
            object_id: self[self.root?].object_id,
        })
    }

    pub fn ingest(&mut self, output: ExecutorOutput) -> Vec<(PlanBoundary, Vec<ResponseBoundaryItem>)> {
        let reservation = &mut self.parts[usize::from(output.id)];
        assert!(reservation.is_empty(), "Part already has data");
        *reservation = output.data_part;
        self.errors.extend(output.errors);
        for update in output.updates {
            self[update.id].fields.extend(update.fields);
        }
        for path in output.error_paths_to_propagate {
            self.propagate_error(&path);
        }
        // The boundary objects are only accessible after we ingested them
        output.boundaries
    }

    // FIXME: this method is improperly used, when pushing an error we need to propagate it which
    // parent callers never do currently. It's a bit tricky to handle that correctly in the
    // Coordinator during the planning phase.
    pub fn push_error(&mut self, error: impl Into<GraphqlError>) {
        self.errors.push(error.into());
    }

    pub fn with_error(mut self, error: impl Into<GraphqlError>) -> Self {
        self.push_error(error);
        self
    }

    pub fn build(self, schema: Arc<Schema>, keys: Arc<ResponseKeys>, metadata: ExecutionMetadata) -> Response {
        Response::Initial(InitialResponse {
            data: ResponseData {
                schema,
                keys,
                root: self.root,
                parts: self.parts,
            },
            errors: self.errors,
            metadata,
        })
    }

    // The path corresponds to place where a plan failed but couldn't go propagate higher as data
    // was in a different part (provided by a parent plan).
    // To correctly propagate error we're finding the last nullable element in the path and make it
    // nullable. If there's nothing, then root will be null.
    fn propagate_error(&mut self, path: &ResponsePath) {
        let Some(root) = self.root else {
            return;
        };
        let mut last_nullable: Option<ResponseValueId> = None;
        let mut previous: Either<ResponseObjectId, ResponseListId> = Either::Left(root);
        for edge in path.iter() {
            let (unique_id, value) = match (previous, edge.unpack()) {
                (Either::Left(object_id), UnpackedResponseEdge::BoundResponseKey(key)) => {
                    let edge = ResponseEdge::from(key);
                    let unique_id = ResponseValueId::ObjectField { object_id, edge };
                    let value = self[object_id].fields.get(&edge);
                    (unique_id, value)
                }
                (Either::Left(object_id), UnpackedResponseEdge::ExtraField(field_id)) => {
                    let edge = ResponseEdge::from(field_id);
                    let unique_id = ResponseValueId::ObjectField { object_id, edge };
                    let value = self[object_id].fields.get(&edge);
                    (unique_id, value)
                }
                (Either::Right(list_id), UnpackedResponseEdge::Index(index)) => {
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
                    previous = Either::Left(*id);
                }
                ResponseValue::List { id, nullable } => {
                    if *nullable {
                        last_nullable = Some(unique_id);
                    }
                    previous = Either::Right(*id);
                }
                _ => break,
            }
        }
        if let Some(last_nullable) = last_nullable {
            match last_nullable {
                ResponseValueId::ObjectField { object_id, edge } => {
                    self[object_id].fields.insert(edge, ResponseValue::Null);
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
        edge: ResponseEdge,
    },
    ListItem {
        list_id: ResponseListId,
        index: usize,
    },
}

pub(crate) struct ExecutorOutput {
    id: ResponseDataPartId,
    data_part: ResponseDataPart,
    errors: Vec<GraphqlError>,
    updates: Vec<ResponseObjectUpdate>,
    error_paths_to_propagate: Vec<ResponsePath>,
    boundaries: Vec<(PlanBoundary, Vec<ResponseBoundaryItem>)>,
}

impl ExecutorOutput {
    pub fn new(id: ResponseDataPartId, boundaries: Vec<PlanBoundary>) -> ExecutorOutput {
        ExecutorOutput {
            id,
            data_part: ResponseDataPart::default(),
            errors: Vec::new(),
            updates: Vec::new(),
            error_paths_to_propagate: Vec::new(),
            boundaries: boundaries.into_iter().map(|plan| (plan, vec![])).collect(),
        }
    }

    pub fn push_update(&mut self, update: ResponseObjectUpdate) {
        self.updates.push(update);
    }

    pub fn push_error(&mut self, error: impl Into<GraphqlError>) {
        self.errors.push(error.into());
    }

    pub fn push_errors(&mut self, errors: impl IntoIterator<Item = GraphqlError>) {
        self.errors.extend(errors);
    }

    pub fn push_error_path_to_propagate(&mut self, path: ResponsePath) {
        self.error_paths_to_propagate.push(path);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl std::ops::Index<PlanBoundaryId> for ExecutorOutput {
    type Output = Vec<ResponseBoundaryItem>;

    fn index(&self, index: PlanBoundaryId) -> &Self::Output {
        &self.boundaries[usize::from(index)].1
    }
}

impl std::ops::IndexMut<PlanBoundaryId> for ExecutorOutput {
    fn index_mut(&mut self, index: PlanBoundaryId) -> &mut Self::Output {
        &mut self.boundaries[usize::from(index)].1
    }
}

pub struct ResponseObjectUpdate {
    pub id: ResponseObjectId,
    pub fields: BTreeMap<ResponseEdge, ResponseValue>,
}
