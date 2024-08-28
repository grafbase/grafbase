mod deserialize;
mod ids;

use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
    sync::Arc,
};

use id_newtypes::IdRange;
pub use ids::*;
use itertools::Either;
use schema::{ObjectDefinitionId, Schema};

use self::deserialize::UpdateSeed;

use super::{
    value::ResponseObjectField, ErrorCode, ExecutedResponse, GraphqlError, InputdResponseObjectSet,
    OutputResponseObjectSets, Response, ResponseData, ResponseEdge, ResponseObject, ResponseObjectRef,
    ResponseObjectSet, ResponseObjectSetId, ResponsePath, ResponseValue, UnpackedResponseEdge,
};
use crate::{
    execution::{ExecutionContext, ExecutionError},
    operation::{LogicalPlanId, PreparedOperation},
    Runtime,
};

pub(crate) struct ResponseDataPart {
    id: ResponseDataPartId,
    objects: Vec<ResponseObject>,
    lists: Vec<ResponseValue>,
}

impl ResponseDataPart {
    fn new(id: ResponseDataPartId) -> Self {
        Self {
            id,
            objects: Vec::new(),
            lists: Vec::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.objects.is_empty() && self.lists.is_empty()
    }
}

pub(crate) struct ResponseBuilder {
    // will be None if an error propagated up to the root.
    pub(super) root: Option<(ResponseObjectId, ObjectDefinitionId)>,
    parts: Vec<ResponseDataPart>,
    errors: Vec<GraphqlError>,
}

// Only supporting additions for the current graph. Deletion are... tricky
// It shouldn't be that difficult to know whether a remaining plan still needs a field after
// execution plan creation. But it's definitely not efficient currently. I think we can at
// least wait until we face actual problems. We're focused on OLTP workloads, so might never
// happen.
impl ResponseBuilder {
    pub fn new(root_object_id: ObjectDefinitionId) -> Self {
        let mut initial_part = ResponseDataPart {
            id: ResponseDataPartId::from(0),
            objects: Vec::new(),
            lists: Vec::new(),
        };
        let root_id = initial_part.push_object(ResponseObject::default());
        Self {
            root: Some((root_id, root_object_id)),
            parts: vec![initial_part],
            errors: Vec::new(),
        }
    }

    pub fn push_root_errors(&mut self, errors: impl IntoIterator<Item = GraphqlError>) {
        self.errors.extend(errors);
        self.root = None;
    }

    pub fn push_error(&mut self, error: impl Into<GraphqlError>) {
        let error = error.into();
        if let Some(path) = error.path.as_ref() {
            self.propagate_error(path);
        }
        self.errors.push(error);
    }

    pub fn new_subgraph_response(
        &mut self,
        logical_plan_id: LogicalPlanId,
        root_response_object_set: Arc<InputdResponseObjectSet>,
        tracked_response_object_set_ids: IdRange<ResponseObjectSetId>,
    ) -> SubgraphResponse {
        let id = ResponseDataPartId::from(self.parts.len());
        // reserving the spot until the actual data is written. It's safe as no one can reference
        // any data in this part before it's added. And a part can only be overwritten if it's
        // empty.
        self.parts.push(ResponseDataPart::new(id));
        SubgraphResponse::new(
            ResponseDataPart::new(id),
            logical_plan_id,
            root_response_object_set,
            tracked_response_object_set_ids,
        )
    }

    pub fn root_response_object(&self) -> Option<ResponseObjectRef> {
        self.root.map(|(response_object_id, object_id)| ResponseObjectRef {
            id: response_object_id,
            path: ResponsePath::default(),
            definition_id: object_id,
        })
    }

    pub fn propagate_execution_error(
        &mut self,
        root_response_object_set: Arc<InputdResponseObjectSet>,
        error: ExecutionError,
        any_edge: ResponseEdge,
        default_fields: Option<Vec<ResponseObjectField>>,
    ) {
        let error = GraphqlError::from(error);
        if let Some(fields) = default_fields {
            for obj_ref in root_response_object_set.iter() {
                self[obj_ref.id].extend(fields.clone());
                // Definitely not ideal (for the client) to have a new error each time in the response.
                // Not exactly sure how we should best deal with it.
                self.errors.push(error.clone().with_path(obj_ref.path.child(any_edge)));
            }
        } else {
            let mut invalidated_paths = Vec::<&[ResponseEdge]>::new();
            for obj_ref in root_response_object_set.iter() {
                if !invalidated_paths.iter().any(|path| obj_ref.path.starts_with(path)) {
                    if let Some(invalidated_path) = self.propagate_error(&obj_ref.path) {
                        self.errors.push(error.clone().with_path(obj_ref.path.child(any_edge)));
                        invalidated_paths.push(invalidated_path);
                    }
                }
            }
        }
    }

    pub fn ingest(
        &mut self,
        subgraph_response: SubgraphResponse,
        any_edge: ResponseEdge,
        default_fields: Option<Vec<ResponseObjectField>>,
    ) -> OutputResponseObjectSets {
        let reservation = &mut self.parts[usize::from(subgraph_response.data.id)];
        assert!(reservation.is_empty(), "Part already has data");
        *reservation = subgraph_response.data;

        let mut invalidated_paths = Vec::<&[ResponseEdge]>::new();
        for (update, obj_ref) in subgraph_response
            .updates
            .into_iter()
            .zip(subgraph_response.root_response_object_set.iter())
        {
            match update {
                UpdateSlot::Reserved => {
                    if let Some(fields) = &default_fields {
                        self[obj_ref.id].extend(fields.clone());
                        // If there isn't any existing error within the response object path,
                        // we create one. Errors without any path are considering to be
                        // execution errors which are also enough.
                        if !subgraph_response.errors.iter().any(|error| {
                            error
                                .path
                                .as_ref()
                                .map(|p| p.starts_with(&obj_ref.path))
                                .unwrap_or(true)
                        }) {
                            self.errors.push(
                                GraphqlError::new(
                                    "Missing data from subgraph",
                                    ErrorCode::SubgraphInvalidResponseError,
                                )
                                .with_path(obj_ref.path.child(any_edge)),
                            )
                        }
                    } else if !invalidated_paths.iter().any(|path| obj_ref.path.starts_with(path)) {
                        if let Some(invalidated_path) = self.propagate_error(&obj_ref.path) {
                            // If there isn't any existing error within the response object path,
                            // we create one. Errors without any path are considering to be
                            // execution errors which are also enough.
                            if !subgraph_response.errors.iter().any(|error| {
                                error
                                    .path
                                    .as_ref()
                                    .map(|p| p.starts_with(&obj_ref.path))
                                    .unwrap_or(true)
                            }) {
                                self.errors.push(
                                    GraphqlError::new(
                                        "Missing data from subgraph",
                                        ErrorCode::SubgraphInvalidResponseError,
                                    )
                                    .with_path(obj_ref.path.child(any_edge)),
                                );
                            }
                            invalidated_paths.push(invalidated_path);
                        }
                    }
                }
                UpdateSlot::Fields(fields) => {
                    self[obj_ref.id].extend(fields);
                }
                UpdateSlot::Error => {
                    if !invalidated_paths.iter().any(|path| obj_ref.path.starts_with(path)) {
                        if let Some(invalidated_path) = self.propagate_error(&obj_ref.path) {
                            invalidated_paths.push(invalidated_path);
                        }
                    }
                }
            }
        }
        self.errors.extend(subgraph_response.errors);

        let mut boundaries = subgraph_response.tracked_response_object_sets;
        if !invalidated_paths.is_empty() {
            boundaries = boundaries
                .into_iter()
                .map(|refs| {
                    refs.into_iter()
                        .filter(|obj| !invalidated_paths.iter().any(|path| obj.path.starts_with(path)))
                        .collect()
                })
                .collect();
        }
        OutputResponseObjectSets {
            ids: subgraph_response.tracked_response_object_set_ids,
            sets: boundaries,
        }
    }

    pub fn build(self, schema: Arc<Schema>, operation: Arc<PreparedOperation>) -> Response {
        Response::Executed(ExecutedResponse {
            data: Some(ResponseData {
                schema,
                operation,
                root: self.root.map(|(id, _)| id),
                parts: self.parts,
            }),
            errors: self.errors,
        })
    }

    // The path corresponds to place where a plan failed but couldn't go propagate higher as data
    // was in a different part (provided by a parent plan).
    // To correctly propagate error we're finding the last nullable element in the path and make it
    // nullable. If there's nothing, then root will be null.
    fn propagate_error<'p>(&mut self, path: &'p ResponsePath) -> Option<&'p [ResponseEdge]> {
        let (root, _) = self.root?;

        let mut last_nullable_path_end = 0;
        let mut last_nullable: Option<ResponseValueId> = None;
        let mut previous: Either<ResponseObjectId, ResponseListId> = Either::Left(root);
        for (i, &edge) in path.iter().enumerate() {
            let (id, value) = match (previous, edge.unpack()) {
                (
                    Either::Left(object_id),
                    UnpackedResponseEdge::BoundResponseKey(_) | UnpackedResponseEdge::ExtraFieldResponseKey(_),
                ) => {
                    let Some(field_position) = self[object_id].field_position(edge) else {
                        // Shouldn't happen but equivalent to null
                        return None;
                    };
                    let id = ResponseValueId::ObjectField {
                        object_id,
                        field_position,
                    };
                    let value = &self[object_id][field_position];
                    (id, value)
                }
                (Either::Right(list_id), UnpackedResponseEdge::Index(index)) => {
                    let id = ResponseValueId::ListItem { list_id, index };
                    let Some(value) = self[list_id].get(index) else {
                        // Shouldn't happen but equivalent to null
                        return None;
                    };
                    (id, value)
                }
                _ => return None,
            };
            if value.is_null() {
                return None;
            }
            match *value {
                ResponseValue::Object {
                    nullable,
                    part_id,
                    index,
                } => {
                    if nullable {
                        last_nullable_path_end = i;
                        last_nullable = Some(id);
                    }
                    previous = Either::Left(ResponseObjectId { part_id, index });
                }
                ResponseValue::List {
                    nullable,
                    part_id,
                    offset,
                    length,
                } => {
                    if nullable {
                        last_nullable_path_end = i;
                        last_nullable = Some(id);
                    }
                    previous = Either::Right(ResponseListId {
                        part_id,
                        offset,
                        length,
                    });
                }
                _ => break,
            }
        }
        if let Some(last_nullable) = last_nullable {
            match last_nullable {
                ResponseValueId::ObjectField {
                    object_id,
                    field_position,
                } => {
                    self[object_id][field_position] = ResponseValue::Null;
                }
                ResponseValueId::ListItem { list_id, index } => {
                    self[list_id][index] = ResponseValue::Null;
                }
            }
        } else {
            self.root = None;
        }
        Some(&path[..last_nullable_path_end])
    }
}

enum ResponseValueId {
    ObjectField {
        object_id: ResponseObjectId,
        field_position: usize,
    },
    ListItem {
        list_id: ResponseListId,
        index: usize,
    },
}

pub(crate) struct SubgraphResponse {
    data: ResponseDataPart,
    logical_plan_id: LogicalPlanId,
    root_response_object_set: Arc<InputdResponseObjectSet>,
    errors: Vec<GraphqlError>,
    updates: Vec<UpdateSlot>,
    tracked_response_object_set_ids: IdRange<ResponseObjectSetId>,
    tracked_response_object_sets: Vec<ResponseObjectSet>,
    on_subgraph_response_data: Vec<u8>,
}

impl SubgraphResponse {
    fn new(
        data: ResponseDataPart,
        logical_plan_id: LogicalPlanId,
        root_response_object_set: Arc<InputdResponseObjectSet>,
        tracked_response_object_set_ids: IdRange<ResponseObjectSetId>,
    ) -> Self {
        Self {
            data,
            logical_plan_id,
            root_response_object_set,
            errors: Vec::new(),
            updates: Vec::new(),
            tracked_response_object_set_ids,
            tracked_response_object_sets: tracked_response_object_set_ids
                .into_iter()
                .map(|_| (Vec::new()))
                .collect(),
            on_subgraph_response_data: Vec::new(),
        }
    }

    /// Executors manipulate the response within a Send future, so we can't use a Rc/RefCell
    /// directly. Only once the executor is ready to write should it use this method.
    pub fn as_mut(&mut self) -> SubgraphResponseRefMut<'_> {
        SubgraphResponseRefMut {
            inner: Rc::new(RefCell::new(self)),
        }
    }

    pub fn subgraph_errors(&self) -> impl Iterator<Item = &GraphqlError> + '_ {
        self.errors.iter().filter(|e| {
            matches!(
                e.code,
                ErrorCode::SubgraphError | ErrorCode::SubgraphInvalidResponseError | ErrorCode::SubgraphRequestError
            )
        })
    }
}

#[derive(Clone)]
pub(crate) struct SubgraphResponseRefMut<'resp> {
    /// We end up writing objects or lists at various step of the de-serialization / query
    /// traversal, so having a RefCell is by far the easiest. We don't need a lock as executor are
    /// not expected to parallelize their work.
    /// The Rc makes it possible to write errors at one place and the data in another.
    inner: Rc<RefCell<&'resp mut SubgraphResponse>>,
}

impl<'resp> SubgraphResponseRefMut<'resp> {
    pub fn next_seed<'ctx, R: Runtime>(&self, ctx: ExecutionContext<'ctx, R>) -> Option<UpdateSeed<'resp>>
    where
        'ctx: 'resp,
    {
        self.next_writer()
            .map(|writer| UpdateSeed::new(ctx, self.inner.borrow().logical_plan_id, writer))
    }

    pub fn next_writer(&self) -> Option<ResponseWriter<'resp>> {
        let index = {
            let mut inner = self.inner.borrow_mut();
            if inner.updates.len() == inner.root_response_object_set.len() {
                return None;
            }
            inner.updates.push(UpdateSlot::Reserved);
            inner.updates.len() - 1
        };
        Some(ResponseWriter {
            index,
            part: self.clone(),
        })
    }

    pub fn get_root_response_object(&self, i: usize) -> Option<Ref<'_, ResponseObjectRef>> {
        Ref::filter_map(self.inner.borrow(), |inner| inner.root_response_object_set.get(i)).ok()
    }

    pub fn push_error(&self, error: impl Into<GraphqlError>) {
        self.inner.borrow_mut().errors.push(error.into());
    }

    pub fn push_errors(&self, errors: Vec<GraphqlError>) {
        self.inner.borrow_mut().errors.extend(errors);
    }

    pub fn add_on_subgraph_response_data(&self, data: Vec<u8>) {
        self.inner.borrow_mut().on_subgraph_response_data = data;
    }
}

pub struct ResponseWriter<'resp> {
    index: usize,
    part: SubgraphResponseRefMut<'resp>,
}

impl<'resp> ResponseWriter<'resp> {
    fn part(&self) -> RefMut<'_, &'resp mut SubgraphResponse> {
        self.part.inner.borrow_mut()
    }

    pub fn root_path(&self) -> ResponsePath {
        RefCell::borrow(&self.part.inner).root_response_object_set[self.index]
            .path
            .clone()
    }

    pub fn push_object(&self, object: ResponseObject) -> ResponseObjectId {
        self.part().data.push_object(object)
    }

    pub fn push_list(&self, value: &[ResponseValue]) -> ResponseListId {
        self.part().data.push_list(value)
    }

    pub fn update_root_object_with(&self, fields: Vec<ResponseObjectField>) {
        self.part().updates[self.index] = UpdateSlot::Fields(fields);
    }

    pub fn propagate_error(&self, error: impl Into<GraphqlError>) {
        let mut part = self.part();
        part.errors.push(error.into());
        part.updates[self.index] = UpdateSlot::Error;
    }

    pub fn continue_error_propagation(&self) {
        self.part().updates[self.index] = UpdateSlot::Error;
    }

    pub fn push_error(&self, error: impl Into<GraphqlError>) {
        self.part().errors.push(error.into());
    }

    pub fn push_response_object(&self, set_id: ResponseObjectSetId, obj: ResponseObjectRef) {
        let mut part = self.part();
        let i = part
            .tracked_response_object_set_ids
            .index_of(set_id)
            .unwrap_or_else(|| unreachable!("{set_id} not in {:?}", part.tracked_response_object_set_ids));
        part.tracked_response_object_sets[i].push(obj);
    }
}

enum UpdateSlot {
    Reserved,
    Fields(Vec<ResponseObjectField>),
    Error,
}
