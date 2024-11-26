use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
    sync::Arc,
};

use crate::{
    execution::ExecutionContext,
    operation::ResponseObjectSetDefinitionId,
    response::{
        ConcreteShapeId, DataPart, GraphqlError, InputResponseObjectSet, ResponseObjectField, ResponseObjectRef,
        ResponseObjectSet, ResponseValueId,
    },
    Runtime,
};

use super::deserialize::UpdateSeed;

pub(crate) struct SubgraphResponse {
    pub(super) data: DataPart,
    shape_id: ConcreteShapeId,
    pub(super) root_response_object_set: Arc<InputResponseObjectSet>,
    pub(super) propagated_null_up_to_root: bool,
    pub(super) propagated_null_up_to_paths: Vec<Vec<ResponseValueId>>,
    pub(super) errors: Vec<GraphqlError>,
    pub(super) subgraph_errors: Vec<GraphqlError>,
    pub(super) updates: Vec<ObjectUpdate>,
    pub(super) response_object_sets: Vec<(ResponseObjectSetDefinitionId, ResponseObjectSet)>,
}

impl SubgraphResponse {
    pub(super) fn new(
        data: DataPart,
        shape_id: ConcreteShapeId,
        root_response_object_set: Arc<InputResponseObjectSet>,
    ) -> Self {
        Self {
            data,
            shape_id,
            root_response_object_set,
            propagated_null_up_to_root: false,
            propagated_null_up_to_paths: Vec::new(),
            errors: Vec::new(),
            subgraph_errors: Vec::new(),
            updates: Vec::new(),
            response_object_sets: Vec::new(),
        }
    }

    /// Executors manipulate the response within a Send future, so we can't use a Rc/RefCell
    /// directly. Only once the executor is ready to write should it use this method.
    pub fn as_mut(&mut self) -> SubgraphResponseRefMut<'_> {
        SubgraphResponseRefMut {
            inner: Rc::new(RefCell::new(self)),
        }
    }

    fn propagate_null(&mut self, path: &[ResponseValueId]) {
        let Some(i) = path.iter().rev().position(|value| value.is_nullable()) else {
            self.propagated_null_up_to_root = true;
            return;
        };
        // we inverted the path.
        let i = path.len() - i - 1;

        // if let Some(value_id) = path.get(i).filter(|value_id| value_id.part_id() == self.data.id) {
        //     self.data.make_inaccessible(*value_id);
        // } else {
        self.propagated_null_up_to_paths.push(path[..(i + 1)].to_vec());
        // }
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
            inner.updates.push(ObjectUpdate::None);
            inner.updates.len() - 1
        };
        Some(ResponseWriter {
            index,
            shared_subgraph_response: self.clone(),
        })
    }

    pub fn get_root_response_object(&self, i: usize) -> Option<Ref<'_, ResponseObjectRef>> {
        Ref::filter_map(self.inner.borrow(), |inner| inner.root_response_object_set.get(i)).ok()
    }

    pub fn push_error(&self, error: impl Into<GraphqlError>) {
        self.inner.borrow_mut().errors.push(error.into());
    }

    pub fn set_subgraph_errors(&self, errors: Vec<GraphqlError>) {
        self.inner.borrow_mut().subgraph_errors = errors;
    }
}

pub(crate) struct ResponseWriter<'resp> {
    index: usize,
    shared_subgraph_response: SubgraphResponseRefMut<'resp>,
}

impl<'resp> ResponseWriter<'resp> {
    fn inner(&self) -> RefMut<'_, &'resp mut SubgraphResponse> {
        self.shared_subgraph_response.inner.borrow_mut()
    }

    pub fn root_object_ref(&self) -> Ref<'_, ResponseObjectRef> {
        Ref::map(self.shared_subgraph_response.inner.borrow(), |part| {
            &part.root_response_object_set[self.index]
        })
    }

    pub fn data(&self) -> RefMut<'_, DataPart> {
        RefMut::map(self.inner(), |inner| &mut inner.data)
    }

    pub fn update_root_object(&self, update: ObjectUpdate) {
        self.inner().updates[self.index] = update;
    }

    pub fn propagate_null(&self, path: &[ResponseValueId]) {
        self.inner().propagate_null(path)
    }

    pub fn push_error(&self, error: impl Into<GraphqlError>) {
        self.inner().errors.push(error.into());
    }

    pub fn push_object_ref(&self, set_id: ResponseObjectSetDefinitionId, obj: ResponseObjectRef) {
        let mut part = self.inner();
        if let Some((_, set)) = part.response_object_sets.iter_mut().find(|(id, _)| set_id == *id) {
            set.push(obj);
        } else {
            part.response_object_sets.push((set_id, vec![obj]));
        }
    }
}

pub(crate) enum ObjectUpdate {
    None,
    Fields(Vec<ResponseObjectField>),
    Error,
}
