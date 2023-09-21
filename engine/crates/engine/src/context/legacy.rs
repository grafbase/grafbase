//! Contexts for resolving the legacy async_graphql derive based stuff...
//!
//! Hopefully one day we can delete this

use engine_parser::{
    types::{Field, SelectionSet},
    Positioned,
};
use ulid::Ulid;

use crate::{
    registry::type_kinds::OutputType, Context, ContextField, QueryEnv, QueryPath, QueryPathSegment, SchemaEnv,
};

#[derive(Clone)]
pub struct ContextSelectionSetLegacy<'a> {
    /// The type our selection set applies to
    pub ty: OutputType<'a>,

    /// The current path being resolved.
    pub path: QueryPath,
    /// The selection set being resolved
    pub item: &'a Positioned<SelectionSet>,
    /// Context scoped to the current schema
    pub schema_env: &'a SchemaEnv,
    /// Context scoped to the current query
    pub query_env: &'a QueryEnv,
}

impl<'a> ContextSelectionSetLegacy<'a> {
    pub fn with_field(&'a self, field: &'a Positioned<Field>) -> ContextField<'a> {
        let meta_field = self.ty.field(&field.node.name.node);

        let mut path = self.path.clone();
        path.push(field.node.response_key().node.as_str());

        ContextField {
            field: meta_field.expect("malformed query that should be validated before here"),
            parent_type: self
                .ty
                .try_into()
                .expect("have to be a selection set if were going into a field"),
            execution_id: Ulid::from_datetime(self.query_env.current_datetime.clone().into()),
            path,
            item: field,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn with_index(&'a self, idx: usize) -> ContextSelectionSetLegacy<'a> {
        let mut path = self.path.clone();
        path.push(QueryPathSegment::Index(idx));
        ContextSelectionSetLegacy {
            ty: self.ty,
            path,
            item: self.item,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }

    /// Makes a ContextSelection for the given selection set.
    ///
    /// This should be used on spreads
    pub fn with_selection_set(
        &self,
        selection_set: &'a Positioned<SelectionSet>,
        new_target: OutputType<'a>,
    ) -> ContextSelectionSetLegacy<'a> {
        ContextSelectionSetLegacy {
            ty: new_target,
            path: self.path.clone(),
            item: selection_set,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }
}

impl<'a> ContextField<'a> {
    pub fn with_selection_set_legacy(
        &self,
        selection_set: &'a Positioned<SelectionSet>,
    ) -> ContextSelectionSetLegacy<'a> {
        ContextSelectionSetLegacy {
            ty: self.field_base_type(),
            path: self.path.clone(),
            item: selection_set,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }
}

impl<'a> Context<'a> for ContextSelectionSetLegacy<'a> {
    fn path(&self) -> &QueryPath {
        &self.path
    }

    fn query_env(&self) -> &'a QueryEnv {
        self.query_env
    }

    fn schema_env(&self) -> &'a SchemaEnv {
        self.schema_env
    }
}
