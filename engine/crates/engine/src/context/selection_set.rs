use std::fmt::{self, Formatter};

use ulid::Ulid;

use crate::{
    parser::types::{Field, SelectionSet},
    query_path::QueryPath,
    registry::{MetaType, TypeReference},
    schema::SchemaEnv,
    ContextField, Positioned, QueryEnv, QueryPathSegment, ResolverChainNode,
};

use super::ContextExt;

#[derive(Clone)]
pub struct ContextSelectionSet<'a> {
    /// The current path being resolved.
    pub path: QueryPath,
    /// The current resolver path being resolved.
    pub resolver_node: Option<ResolverChainNode<'a>>,
    /// The selection set being resolved
    pub item: &'a Positioned<SelectionSet>,
    /// Context scoped to the current schema
    pub schema_env: &'a SchemaEnv,
    /// Context scoped to the current query
    pub query_env: &'a QueryEnv,
}

impl<'a> ContextSelectionSet<'a> {
    /// We add a new field with the Context with the proper execution_id generated.
    pub fn with_field(
        &'a self,
        field: &'a Positioned<Field>,
        ty: Option<&'a MetaType>,
        selections: Option<&'a SelectionSet>,
    ) -> ContextField<'a> {
        let registry = &self.schema_env.registry;

        let meta_field = ty.and_then(|ty| ty.field_by_name(&field.node.name.node));

        let meta = meta_field.and_then(|field| registry.types.get(field.ty.named_type().as_str()));

        let mut path = self.path.clone();
        path.push(field.node.response_key().node.as_str());

        ContextField {
            resolver_node: Some(ResolverChainNode {
                parent: self.resolver_node.as_ref(),
                segment: path.last().unwrap().clone(),
                ty: meta,
                field: meta_field,
                executable_field: Some(field),
                resolver: meta_field.map(|x| &x.resolver),
                execution_id: Ulid::from_datetime(self.query_env.current_datetime.clone().into()),
                selections,
            }),
            path,
            item: field,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn with_index(&'a self, idx: usize, selections: Option<&'a SelectionSet>) -> ContextSelectionSet<'a> {
        let mut path = self.path.clone();
        path.push(QueryPathSegment::Index(idx));
        ContextSelectionSet {
            resolver_node: Some(ResolverChainNode {
                parent: self.resolver_node.as_ref(),
                segment: path.last().cloned().unwrap(),
                field: self.resolver_node.as_ref().and_then(|x| x.field),
                executable_field: self.resolver_node.as_ref().and_then(|x| x.executable_field),
                ty: self.resolver_node.as_ref().and_then(|x| x.ty),
                resolver: self.resolver_node.as_ref().and_then(|x| x.resolver),
                execution_id: Ulid::from_datetime(self.query_env.current_datetime.clone().into()),
                selections,
            }),
            path,
            item: self.item,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }

    #[doc(hidden)]
    pub fn with_selection_set(&self, selection_set: &'a Positioned<SelectionSet>) -> ContextSelectionSet<'a> {
        ContextSelectionSet {
            path: self.path.clone(),
            resolver_node: self.resolver_node.clone(),
            item: selection_set,
            schema_env: self.schema_env,
            query_env: self.query_env,
        }
    }
}

impl ContextExt for ContextSelectionSet<'_> {
    fn path(&self) -> &QueryPath {
        &self.path
    }

    fn query_env(&self) -> &QueryEnv {
        self.query_env
    }

    fn schema_env(&self) -> &SchemaEnv {
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
