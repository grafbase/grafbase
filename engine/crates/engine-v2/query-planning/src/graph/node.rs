use bitflags::bitflags;
use std::borrow::Cow;

use schema::{FieldDefinitionId, ResolverDefinitionId, Schema};
use walker::Walk as _;

use crate::{dot_graph::Attrs, Operation};

use super::OperationGraph;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node<F> {
    /// Root node, unique
    Root,
    /// Field in the operation, or an extra one to satisfy requirements
    QueryField(QueryField<F>),
    /// Defines how to access data from a subgraph
    Resolver(Resolver),
    /// Field that can be provided by a resolver with extra metadata such as field's @provides
    /// merged parent @provides ones. It's used to mark a QueryField as providable by a resolver
    /// simply by its existence. And while adding requirements it's used to know whether a resolver
    /// could provide it either because it's simply part of the subgraph or part of a field's
    /// @provides.
    ProvidableField(ProvidableField),
}

impl<F: Copy> Node<F> {
    /// Meant to be as readable as possible for large graphs with colors.
    pub(crate) fn label<'a, Op: Operation<FieldId = F>>(&self, graph: &OperationGraph<'a, Op>) -> Cow<'a, str> {
        match self {
            Node::Root => "root".into(),
            Node::QueryField(field) => format!(
                "{}{}",
                if field.is_extra() { "*" } else { "" },
                graph.operation.field_label(field.id)
            )
            .into(),
            Node::ProvidableField(field) => format!(
                "{}@{}",
                field.field_definition_id.walk(graph.schema).name(),
                field.resolver_definition_id.walk(graph.schema).name()
            )
            .into(),
            Node::Resolver(resolver) => resolver.definition_id.walk(graph.schema).name(),
        }
    }

    /// Meant to be as readable as possible for large graphs with colors.
    pub(crate) fn pretty_label<'a, Op: Operation<FieldId = F>>(&self, graph: &OperationGraph<'a, Op>) -> Attrs<'a> {
        let attrs = Attrs::label(self.label(graph));
        match self {
            Node::ProvidableField(_) => attrs.with("shape=box").with("color=dodgerblue"),
            Node::Resolver(_) => attrs.with("shape=parallelogram").with("color=dodgerblue"),
            _ => attrs,
        }
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct FieldFlags: u8 {
        /// Extra field that is not part of the operation and should not be returned to the user.
        const EXTRA = 1;
        /// Defines whether a field must be requested from the subgraphs. Operations fields are
        /// obviously indispensable, but fields necessary for @authorized also are for example.
        const INDISPENSABLE = 1 << 1;
        /// Whether the field is a scalar/leaf node.
        const SCALAR = 1 << 2;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryField<Id> {
    pub id: Id,
    pub(crate) flags: FieldFlags,
}

impl<Id> QueryField<Id> {
    pub fn is_indispensable(&self) -> bool {
        self.flags.contains(FieldFlags::INDISPENSABLE)
    }

    pub fn is_extra(&self) -> bool {
        self.flags.contains(FieldFlags::EXTRA)
    }

    pub fn is_scalar(&self) -> bool {
        self.flags.contains(FieldFlags::SCALAR)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Resolver {
    pub definition_id: ResolverDefinitionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProvidableField {
    pub resolver_definition_id: ResolverDefinitionId,
    pub field_definition_id: FieldDefinitionId,
}

impl ProvidableField {
    pub(crate) fn child(&self, schema: &Schema, field_definition_id: FieldDefinitionId) -> Option<ProvidableField> {
        let resolver_definition = self.resolver_definition_id.walk(schema);
        if resolver_definition.can_provide(field_definition_id) {
            Some(ProvidableField {
                resolver_definition_id: self.resolver_definition_id,
                field_definition_id,
            })
        } else {
            None
        }
    }
}

impl<F> Node<F> {
    pub fn as_resolver(&self) -> Option<&Resolver> {
        match self {
            Node::Resolver(r) => Some(r),
            _ => None,
        }
    }

    pub fn as_providable_field(&self) -> Option<&ProvidableField> {
        match self {
            Node::ProvidableField(r) => Some(r),
            _ => None,
        }
    }

    pub fn as_query_field(&self) -> Option<&QueryField<F>> {
        match self {
            Node::QueryField(field) => Some(field),
            _ => None,
        }
    }
}
