use std::cell::{Cell, Ref, RefCell, RefMut};

use crate::{
    prepare::{
        CachedOperationContext, DataOrLookupFieldId, FieldShapeRecord, OperationPlanContext, PreparedOperation,
        RootFieldsShape, RootFieldsShapeId,
    },
    response::{ResponsePartBuilder, ResponseValueId},
};
use bytes::Bytes;
use operation::ResponseKeys;
use schema::Schema;
use walker::Walk as _;

pub(crate) struct SeedState<'ctx, 'parent> {
    pub schema: &'ctx Schema,
    pub operation: &'ctx PreparedOperation,
    pub root_shape: RootFieldsShape<'ctx>,

    // -- mutable parts --
    // Range isn't copy...
    borrowable_bytes_range: Cell<AddressRange>,
    pub bubbling_up_deser_error: Cell<bool>,
    pub response: RefCell<ResponsePartBuilder<'ctx>>,
    pub(super) parent_path: Cell<&'parent [ResponseValueId]>,
    pub(super) local_path: RefCell<Vec<ResponseValueId>>,
}

#[derive(Clone, Copy)]
struct AddressRange {
    start: *const u8,
    end: *const u8,
}

/// SAFETY: This type only contains raw pointers representing addresses, not used in any other
/// fashion.
unsafe impl Send for AddressRange {}

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
            borrowable_bytes_range: Cell::new(AddressRange {
                start: std::ptr::null(),
                end: std::ptr::null(),
            }),
            response: RefCell::new(response_part),
            bubbling_up_deser_error: Default::default(),
            local_path: Default::default(),
            parent_path: Default::default(),
        }
    }

    // Any deserialization that use those bytes will be able to keep references for strings rather
    // than create new owned strings.
    pub fn push_borrowable_bytes(&self, source: Bytes) {
        let start = source.as_ptr();
        // SAFETY: Just pointer arithmetics to compute the end of the memory section the source
        // points to.
        let end = unsafe { start.add(source.len()) };
        self.borrowable_bytes_range.set(AddressRange { start, end });
        self.response.borrow_mut().data.keep_bytes(source);
    }

    pub fn can_be_borrowed(&self, s: &str) -> bool {
        let start = s.as_ptr();
        // SAFETY: Just pointer arithmetics to compute the end of the memory section the reference
        // points to.
        let end = unsafe { start.add(s.len()) };
        let range = self.borrowable_bytes_range.get();
        // If the reference is within the range of the borrowable bytes, we can keep it as
        // borrowed.
        range.start <= start && end <= range.end
    }

    pub fn into_response_part(self) -> ResponsePartBuilder<'ctx> {
        self.response.into_inner()
    }

    pub fn display_path(&self) -> impl std::fmt::Display + '_ {
        DisplayPath {
            keys: self.response_keys(),
            parent_path: self.parent_path.get(),
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
    parent_path: &'a [ResponseValueId],
    path: Ref<'a, Vec<ResponseValueId>>,
}

impl std::fmt::Display for DisplayPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_path(f, self.keys, self.parent_path.iter().copied())?;
        if !self.path.is_empty() {
            f.write_str(".")?;
            write_path(f, self.keys, self.path.iter().copied())
        } else {
            Ok(())
        }
    }
}

fn write_path(
    f: &mut std::fmt::Formatter<'_>,
    keys: &ResponseKeys,
    values: impl IntoIterator<Item = ResponseValueId>,
) -> std::fmt::Result {
    use std::fmt::Display as _;
    let mut values = values.into_iter();
    if let Some(first) = values.next() {
        match first {
            ResponseValueId::Field { key, .. } => {
                f.write_str(&keys[key])?;
            }
            ResponseValueId::Index { index, .. } => {
                index.fmt(f)?;
            }
        }
        for value in values {
            f.write_str(".")?;
            match value {
                ResponseValueId::Field { key, .. } => {
                    f.write_str(&keys[key])?;
                }
                ResponseValueId::Index { index, .. } => {
                    index.fmt(f)?;
                }
            }
        }
    }
    Ok(())
}
