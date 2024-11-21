mod deserialize;
mod ids;

use std::{
    cell::{Ref, RefCell, RefMut},
    cmp::Ordering,
    collections::VecDeque,
    rc::Rc,
    sync::Arc,
};

use grafbase_telemetry::graphql::GraphqlResponseStatus;
pub use ids::*;
use itertools::Either;
use schema::{ObjectDefinitionId, Schema};

use self::deserialize::UpdateSeed;

use super::{
    value::ResponseObjectField, ConcreteShapeId, ErrorCode, ErrorCodeCounter, ExecutedResponse, GraphqlError,
    InputResponseObjectSet, OutputResponseObjectSets, PositionedResponseKey, Response, ResponseData, ResponseEdge,
    ResponseObject, ResponseObjectRef, ResponseObjectSet, ResponsePath, ResponseValue, UnpackedResponseEdge,
};
use crate::{
    execution::{ExecutionContext, ExecutionError},
    operation::ResponseObjectSetDefinitionId,
    prepare::CachedOperation,
    utils::BufferPool,
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

    pub fn push_error(&mut self, error: impl Into<GraphqlError>) {
        let error: GraphqlError = error.into();
        if let Some(path) = error.path.as_ref() {
            self.propagate_error(path);
        }
        self.errors.push(error);
    }

    pub fn new_subgraph_response(
        &mut self,
        shape_id: ConcreteShapeId,
        root_response_object_set: Arc<InputResponseObjectSet>,
    ) -> SubgraphResponse {
        let id = ResponseDataPartId::from(self.parts.len());

        // reserving the spot until the actual data is written. It's safe as no one can reference
        // any data in this part before it's added. And a part can only be overwritten if it's
        // empty.
        self.parts.push(ResponseDataPart::new(id));

        SubgraphResponse::new(ResponseDataPart::new(id), shape_id, root_response_object_set)
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
        root_response_object_set: Arc<InputResponseObjectSet>,
        error: ExecutionError,
        any_field_key: PositionedResponseKey,
        default_fields: Option<Vec<ResponseObjectField>>,
    ) {
        let error = GraphqlError::from(error);
        if let Some(fields) = default_fields {
            for obj_ref in root_response_object_set.iter() {
                self[obj_ref.id].extend(fields.clone());
                // Definitely not ideal (for the client) to have a new error each time in the response.
                // Not exactly sure how we should best deal with it.
                self.errors
                    .push(error.clone().with_path(obj_ref.path.child(any_field_key)));
            }
        } else {
            let mut invalidated_paths = Vec::<&[ResponseEdge]>::new();
            for obj_ref in root_response_object_set.iter() {
                if !invalidated_paths.iter().any(|path| obj_ref.path.starts_with(path)) {
                    if let Some(invalidated_path) = self.propagate_error(&obj_ref.path) {
                        self.errors
                            .push(error.clone().with_path(obj_ref.path.child(any_field_key)));
                        invalidated_paths.push(invalidated_path);
                    }
                }
            }
        }
    }

    pub fn ingest(
        &mut self,
        subgraph_response: SubgraphResponse,
        any_field_key: PositionedResponseKey,
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
                                .with_path(obj_ref.path.child(any_field_key)),
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
                                    .with_path(obj_ref.path.child(any_field_key)),
                                );
                            }
                            invalidated_paths.push(invalidated_path);
                        }
                    }
                }
                UpdateSlot::Fields(mut fields) => {
                    fields.sort_unstable_by(|a, b| a.key.cmp(&b.key));
                    self.recursive_merge_object(obj_ref.id, fields);
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

        let mut boundaries = subgraph_response.response_object_sets;
        if !invalidated_paths.is_empty() {
            boundaries = boundaries
                .into_iter()
                .map(|(id, refs)| {
                    (
                        id,
                        refs.into_iter()
                            .filter(|obj| !invalidated_paths.iter().any(|path| obj.path.starts_with(path)))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect();
        }
        let (ids, sets) = boundaries.into_iter().unzip();
        OutputResponseObjectSets { ids, sets }
    }

    fn recursive_merge_object(
        &mut self,
        object_id: ResponseObjectId,
        new_fields_sorted_by_edge: Vec<ResponseObjectField>,
    ) {
        let mut new_fields_sorted_by_edge = VecDeque::from(new_fields_sorted_by_edge);
        let Some(mut new_field) = new_fields_sorted_by_edge.pop_front() else {
            return;
        };

        let mut existing_fields = std::mem::take(&mut self[object_id].fields_sorted_by_key);
        let n = existing_fields.len();
        let mut i = 0;
        loop {
            if i >= n {
                existing_fields.push(new_field);
                break;
            }
            // SAFETY we only push new elements and we compare against the initial size n. So i is
            // guaranteed to be within the array.
            let existing_field = unsafe { existing_fields.get_unchecked(i) };
            match existing_field.key.cmp(&new_field.key) {
                Ordering::Less => {
                    i += 1;
                }
                Ordering::Greater => {
                    // Adding at the end and will be sorted later.
                    existing_fields.push(new_field);
                    if let Some(next) = new_fields_sorted_by_edge.pop_front() {
                        new_field = next;
                    } else {
                        break;
                    }
                }
                Ordering::Equal => {
                    self.recursive_merge_value(existing_field.value.clone(), new_field.value);
                    i += 1;
                    if let Some(next) = new_fields_sorted_by_edge.pop_front() {
                        new_field = next;
                    } else {
                        break;
                    }
                }
            }
        }
        existing_fields.append(&mut Vec::from(new_fields_sorted_by_edge));
        existing_fields.sort_unstable_by(|a, b| a.key.cmp(&b.key));

        self[object_id].fields_sorted_by_key = existing_fields;
    }

    fn recursive_merge_value(&mut self, existing: ResponseValue, new: ResponseValue) {
        match (existing, new) {
            (
                ResponseValue::Object {
                    part_id: existing_part_id,
                    index: existing_index,
                    ..
                },
                ResponseValue::Object {
                    part_id: new_part_id,
                    index: new_index,
                    ..
                },
            ) => {
                let existing_object_id = ResponseObjectId {
                    part_id: existing_part_id,
                    index: existing_index,
                };
                let new_object_id = ResponseObjectId {
                    part_id: new_part_id,
                    index: new_index,
                };
                let new_fields_sorted_by_edge = std::mem::take(&mut self[new_object_id].fields_sorted_by_key);
                self.recursive_merge_object(existing_object_id, new_fields_sorted_by_edge);
            }
            (
                ResponseValue::List {
                    part_id: existing_part_id,
                    offset: existing_offset,
                    length: existing_length,
                    ..
                },
                ResponseValue::List {
                    part_id: new_part_id,
                    offset: new_offset,
                    length: new_length,
                    ..
                },
            ) => {
                let existing_list_id = ResponseListId {
                    part_id: existing_part_id,
                    offset: existing_offset,
                    length: existing_length,
                };
                let new_list_id = ResponseListId {
                    part_id: new_part_id,
                    offset: new_offset,
                    length: new_length,
                };
                self.recursive_merge_list(existing_list_id, new_list_id)
            }
            _ => {
                // FIXME: Unlikely, but we should generate an error here.
                tracing::error!("Trying to merge something values that aren't a couple of objects/list");
            }
        }
    }

    fn recursive_merge_list(&mut self, existing_list_id: ResponseListId, new_list_id: ResponseListId) {
        let mut i = 0;
        while let Some((existing, new)) = self[existing_list_id].get(i).zip(self[new_list_id].get(i)) {
            self.recursive_merge_value(existing.clone(), new.clone());
            i += 1;
        }
        // FIXME: Unlikely but should generate an error here.
    }

    pub fn graphql_status(&self) -> GraphqlResponseStatus {
        if self.errors.is_empty() {
            GraphqlResponseStatus::Success
        } else {
            GraphqlResponseStatus::FieldError {
                count: self.errors.len() as u64,
                data_is_null: self.root.is_none(),
            }
        }
    }

    pub fn build<OnOperationResponseHookOutput>(
        self,
        schema: Arc<Schema>,
        operation: Arc<CachedOperation>,
        on_operation_response_output: OnOperationResponseHookOutput,
    ) -> Response<OnOperationResponseHookOutput> {
        let error_code_counter = ErrorCodeCounter::from_errors(&self.errors);
        Response::Executed(ExecutedResponse {
            operation,
            data: self.root.map(|(root, _)| ResponseData {
                schema,
                root,
                parts: self.parts,
            }),
            errors: self.errors,
            error_code_counter,
            on_operation_response_output: Some(on_operation_response_output),
            extensions: None,
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
    shape_id: ConcreteShapeId,
    root_response_object_set: Arc<InputResponseObjectSet>,
    errors: Vec<GraphqlError>,
    updates: Vec<UpdateSlot>,
    response_object_sets: Vec<(ResponseObjectSetDefinitionId, ResponseObjectSet)>,
    buffers: BufferPool<ResponseValue>,
}

impl SubgraphResponse {
    fn new(
        data: ResponseDataPart,
        shape_id: ConcreteShapeId,
        root_response_object_set: Arc<InputResponseObjectSet>,
    ) -> Self {
        Self {
            data,
            shape_id,
            root_response_object_set,
            errors: Vec::new(),
            updates: Vec::new(),
            response_object_sets: Vec::new(),
            buffers: Default::default(),
        }
    }

    /// Executors manipulate the response within a Send future, so we can't use a Rc/RefCell
    /// directly. Only once the executor is ready to write should it use this method.
    pub fn as_mut(&mut self) -> SubgraphResponseRefMut<'_> {
        SubgraphResponseRefMut {
            inner: Rc::new(RefCell::new(self)),
        }
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
    pub fn next_seed<'ctx, R: Runtime>(&self, ctx: &ExecutionContext<'ctx, R>) -> Option<UpdateSeed<'resp>>
    where
        'ctx: 'resp,
    {
        self.next_writer()
            .map(|writer| UpdateSeed::new(*ctx, self.inner.borrow().shape_id, writer))
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

    pub fn new_list(&self) -> Vec<ResponseValue> {
        self.part().buffers.pop()
    }

    // Create a Vec with `new_list` before to re-use an existing Vec.
    pub fn push_list(&self, mut values: Vec<ResponseValue>) -> ResponseListId {
        let mut part = self.part();
        let id = part.data.push_list(&mut values);
        part.buffers.push(values);
        id
    }

    pub fn push_empty_list(&self) -> ResponseListId {
        self.part().data.push_list(&mut Vec::new())
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

    pub fn push_response_object(&self, set_id: ResponseObjectSetDefinitionId, obj: ResponseObjectRef) {
        let mut part = self.part();
        if let Some((_, set)) = part.response_object_sets.iter_mut().find(|(id, _)| set_id == *id) {
            set.push(obj);
        } else {
            part.response_object_sets.push((set_id, vec![obj]));
        }
    }
}

enum UpdateSlot {
    Reserved,
    Fields(Vec<ResponseObjectField>),
    Error,
}
