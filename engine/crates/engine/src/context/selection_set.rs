use std::fmt::{self, Formatter};

use query_path::QueryPath;
use ulid::Ulid;

use super::ext::Context;
use crate::{
    parser::types::{Field, SelectionSet},
    registry::type_kinds::SelectionSetTarget,
    schema::SchemaEnv,
    ContextField, Positioned, QueryEnv,
};

#[derive(Clone)]
pub struct ContextSelectionSet<'a> {
    /// The type our selection set applies to
    pub ty: SelectionSetTarget<'a>,

    /// The current path being resolved.
    pub path: QueryPath,
    /// The selection set being resolved
    pub item: &'a Positioned<SelectionSet>,
    /// Context scoped to the current schema
    pub schema_env: &'a SchemaEnv,
    /// Context scoped to the current query
    pub query_env: &'a QueryEnv,
}

impl<'a> ContextSelectionSet<'a> {
    /// We add a new field with the Context with the proper execution_id generated.
    pub fn with_field(&'a self, field: &'a Positioned<Field>) -> ContextField<'a> {
        let meta_field = self.ty.field(&field.node.name.node);

        let mut path = self.path.clone();
        path.push(field.node.response_key().node.as_str());

        ContextField {
            field: meta_field.expect("query to be validated before this point"),
            parent_type: self.ty,
            execution_id: Ulid::from_datetime(self.query_env.current_datetime.clone().into()),
            path,
            item: field,
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
        new_target: SelectionSetTarget<'a>,
    ) -> ContextSelectionSet<'a> {
        ContextSelectionSet {
            ty: new_target,
            path: self.path.clone(),
            item: selection_set,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }
}

impl<'a> Context<'a> for ContextSelectionSet<'a> {
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

impl std::fmt::Debug for ContextSelectionSet<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContextSelectionSet")
            .field("path", &self.path)
            .field("item", &self.item)
            .finish()
    }
}
