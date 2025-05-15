use std::cell::{Cell, Ref, RefCell, RefMut};

use crate::{
    prepare::{
        CachedOperationContext, DataOrLookupFieldId, FieldShapeRecord, OperationPlanContext, PreparedOperation,
        RootFieldsShape, RootFieldsShapeId,
    },
    response::{ResponseObjectRef, ResponsePartBuilder, ResponseValueId},
};
use itertools::Itertools;
use operation::ResponseKeys;
use schema::Schema;
use walker::Walk as _;

use super::{BatchRootFieldsSeed, RootFieldsSeed};

pub(crate) struct SeedState<'ctx, 'parent> {
    pub schema: &'ctx Schema,
    pub operation: &'ctx PreparedOperation,
    pub root_shape: RootFieldsShape<'ctx>,

    // -- mutable parts --
    pub bubbling_up_deser_error: Cell<bool>,
    pub response: RefCell<ResponsePartBuilder<'ctx>>,
    pub(super) parent_path: Cell<&'parent [ResponseValueId]>,
    pub(super) local_path: RefCell<Vec<ResponseValueId>>,
}

impl<'ctx> From<&SeedState<'ctx, '_>> for CachedOperationContext<'ctx> {
    fn from(state: &SeedState<'ctx, '_>) -> Self {
        CachedOperationContext {
            schema: state.schema,
            cached: &state.operation.cached,
        }
    }
}

impl<'ctx> From<&SeedState<'ctx, '_>> for OperationPlanContext<'ctx> {
    fn from(state: &SeedState<'ctx, '_>) -> Self {
        OperationPlanContext {
            schema: state.schema,
            cached: &state.operation.cached,
            plan: &state.operation.plan,
        }
    }
}

impl<'ctx> From<&SeedState<'ctx, '_>> for &'ctx Schema {
    fn from(state: &SeedState<'ctx, '_>) -> Self {
        state.schema
    }
}

impl<'ctx, 'parent> SeedState<'ctx, 'parent> {
    pub fn new(response_part: ResponsePartBuilder<'ctx>, shape_id: RootFieldsShapeId) -> Self {
        let schema = response_part.schema;
        let operation = response_part.operation;
        let root_shape = shape_id.walk((schema, operation));
        SeedState {
            schema,
            operation,
            root_shape,
            response: RefCell::new(response_part),
            bubbling_up_deser_error: Default::default(),
            local_path: Default::default(),
            parent_path: Default::default(),
        }
    }

    pub fn parent_list_seed<ParentObjects>(
        &self,
        parent_objects: ParentObjects,
    ) -> BatchRootFieldsSeed<'ctx, 'parent, '_, ParentObjects>
    where
        ParentObjects: IntoIterator<Item = &'parent ResponseObjectRef>,
    {
        BatchRootFieldsSeed {
            state: self,
            parent_objects,
        }
    }

    pub fn parent_seed(&self, parent_object: &'parent ResponseObjectRef) -> RootFieldsSeed<'ctx, 'parent, '_> {
        RootFieldsSeed {
            state: self,
            parent_object,
        }
    }

    pub fn into_response_part(self) -> ResponsePartBuilder<'ctx> {
        self.response.into_inner()
    }

    pub fn display_path(&self) -> impl std::fmt::Display + '_ {
        DisplayPath {
            keys: self.response_keys(),
            path: self.local_path.borrow(),
        }
    }

    pub(super) fn reset(&self, path: &'parent [ResponseValueId]) {
        debug_assert!(self.local_path.borrow().is_empty());
        self.bubbling_up_deser_error.set(false);
        self.parent_path.set(path);
    }

    pub(super) fn response_keys(&self) -> &'ctx ResponseKeys {
        &self.operation.cached.operation.response_keys
    }

    pub(super) fn local_path_mut(&self) -> RefMut<'_, Vec<ResponseValueId>> {
        self.local_path.borrow_mut()
    }

    pub(super) fn path(&self) -> (&[ResponseValueId], Ref<'_, Vec<ResponseValueId>>) {
        (self.parent_path.get(), self.local_path.borrow())
    }

    pub(super) fn should_report_error_for(&self, field: &FieldShapeRecord) -> bool {
        field.query_position_before_modifications.is_some()
            && match field.id {
                DataOrLookupFieldId::Data(id) => {
                    self.operation.plan.query_modifications.included_response_data_fields[id]
                }
                DataOrLookupFieldId::Lookup(_) => false,
            }
    }
}

struct DisplayPath<'a> {
    keys: &'a ResponseKeys,
    path: Ref<'a, Vec<ResponseValueId>>,
}

impl std::fmt::Display for DisplayPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            self.path.iter().format_with(".", |value_id, f| match value_id {
                ResponseValueId::Field { key, .. } => f(&format_args!("{}", &self.keys[*key])),
                ResponseValueId::Index { index, .. } => f(&format_args!("{}", index)),
            }),
        ))
    }
}
