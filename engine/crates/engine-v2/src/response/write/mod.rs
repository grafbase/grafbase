mod deserialize;
mod ids;

use std::{collections::BTreeMap, sync::Arc};

pub(crate) use deserialize::SeedContext;
pub use ids::*;
use itertools::Either;
use schema::{ObjectId, Schema};

use super::{
    ExecutionMetadata, GraphqlError, InitialResponse, ResponseBoundaryItem, ResponseData, ResponseEdge, ResponseObject,
    ResponsePath, ResponseValue, UnpackedResponseEdge,
};
use crate::{
    plan::{OperationPlan, PlanBoundaryId},
    utils::IdRange,
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
    pub(super) root: Option<(ResponseObjectId, ObjectId)>,
    parts: Vec<ResponseDataPart>,
    errors: Vec<GraphqlError>,
}

// Only supporting additions for the current graph. Deletion are... tricky
// It shouldn't be that difficult to know whether a remaining plan still needs a field after
// execution plan creation. But it's definitely not efficient currently. I think we can at
// least wait until we face actual problems. We're focused on OLTP workloads, so might never
// happen.
impl ResponseBuilder {
    pub fn new(root_object_id: ObjectId) -> Self {
        let mut builder = ResponsePart::new(ResponseDataPartId::from(0), IdRange::empty());
        let root_id = builder.push_object(ResponseObject {
            fields: BTreeMap::new(),
        });
        Self {
            root: Some((root_id, root_object_id)),
            parts: vec![builder.data],
            errors: vec![],
        }
    }

    pub fn new_part(&mut self, boundary_ids: IdRange<PlanBoundaryId>) -> ResponsePart {
        let id = ResponseDataPartId::from(self.parts.len());
        // reserving the spot until the actual data is written. It's safe as no one can reference
        // any data in this part before it's added. And a part can only be overwritten if it's
        // empty.
        self.parts.push(ResponseDataPart::default());
        ResponsePart::new(id, boundary_ids)
    }

    pub fn root_response_boundary_item(&self) -> Option<ResponseBoundaryItem> {
        self.root.map(|(response_object_id, object_id)| ResponseBoundaryItem {
            response_object_id,
            response_path: ResponsePath::default(),
            object_id,
        })
    }

    pub fn ingest(&mut self, output: ResponsePart) -> Vec<(PlanBoundaryId, Vec<ResponseBoundaryItem>)> {
        let reservation = &mut self.parts[usize::from(output.id)];
        assert!(reservation.is_empty(), "Part already has data");
        *reservation = output.data;
        self.errors.extend(output.errors);
        for update in output.updates {
            self[update.id].fields.extend(update.fields);
        }
        for path in output.error_paths_to_propagate {
            self.propagate_error(&path);
        }
        // The boundary objects are only accessible after we ingested them
        output.plan_boundaries
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

    pub fn build(self, schema: Arc<Schema>, operation: Arc<OperationPlan>, metadata: ExecutionMetadata) -> Response {
        Response::Initial(InitialResponse {
            data: ResponseData {
                schema,
                operation,
                root: self.root.map(|(id, _)| id),
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
        let Some((root, _)) = self.root else {
            return;
        };
        let mut last_nullable: Option<ResponseValueId> = None;
        let mut previous: Either<ResponseObjectId, ResponseListId> = Either::Left(root);
        for &edge in path.iter() {
            let (unique_id, value) = match (previous, edge.unpack()) {
                (
                    Either::Left(object_id),
                    UnpackedResponseEdge::BoundResponseKey(_) | UnpackedResponseEdge::ExtraField(_),
                ) => {
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

pub(crate) struct ResponsePart {
    id: ResponseDataPartId,
    data: ResponseDataPart,
    errors: Vec<GraphqlError>,
    updates: Vec<ResponseObjectUpdate>,
    error_paths_to_propagate: Vec<ResponsePath>,
    plan_boundary_ids_start: usize,
    plan_boundaries: Vec<(PlanBoundaryId, Vec<ResponseBoundaryItem>)>,
}

impl ResponsePart {
    pub fn new(id: ResponseDataPartId, plan_boundary_ids: IdRange<PlanBoundaryId>) -> ResponsePart {
        ResponsePart {
            id,
            data: ResponseDataPart::default(),
            errors: Vec::new(),
            updates: Vec::new(),
            error_paths_to_propagate: Vec::new(),
            plan_boundary_ids_start: usize::from(plan_boundary_ids.start),
            plan_boundaries: plan_boundary_ids.iter().map(|id| (id, Vec::new())).collect(),
        }
    }

    pub fn push_update(&mut self, update: ResponseObjectUpdate) {
        self.updates.push(update);
    }

    pub fn push_error(&mut self, error: impl Into<GraphqlError>) {
        self.errors.push(error.into());
    }

    pub fn push_error_path_to_propagate(&mut self, path: ResponsePath) {
        self.error_paths_to_propagate.push(path);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// This does not change how errors were propagated.
    pub fn replace_errors(&mut self, errors: Vec<GraphqlError>) {
        self.errors = errors;
    }

    pub fn transform_last_object_as_update_for(&mut self, id: ResponseObjectId) {
        if let Some(object) = self.data.objects.pop() {
            self.updates.push(ResponseObjectUpdate {
                id,
                fields: object.fields,
            });
        }
    }
}

impl std::ops::Index<PlanBoundaryId> for ResponsePart {
    type Output = Vec<ResponseBoundaryItem>;

    fn index(&self, id: PlanBoundaryId) -> &Self::Output {
        let n = usize::from(id) - self.plan_boundary_ids_start;
        &self.plan_boundaries[n].1
    }
}

impl std::ops::IndexMut<PlanBoundaryId> for ResponsePart {
    fn index_mut(&mut self, id: PlanBoundaryId) -> &mut Self::Output {
        let n = usize::from(id) - self.plan_boundary_ids_start;
        &mut self.plan_boundaries[n].1
    }
}

pub struct ResponseObjectUpdate {
    pub id: ResponseObjectId,
    pub fields: BTreeMap<ResponseEdge, ResponseValue>,
}
