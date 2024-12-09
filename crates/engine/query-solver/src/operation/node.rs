use std::borrow::Cow;

use schema::{EntityDefinitionId, FieldSetRecord, ResolverDefinitionId, SubgraphId};
use walker::Walk as _;

use crate::{dot_graph::Attrs, FieldFlags, Operation};

use super::OperationGraph;

#[derive(Debug, Clone)]
pub(crate) enum Node<'ctx, FieldId> {
    /// Root node, unique
    Root,
    /// Field in the operation, or an extra one to satisfy requirements
    QueryField(QueryField<FieldId>),
    /// Defines how to access data from a subgraph
    Resolver(Resolver),
    /// Field that can be provided by a resolver with extra metadata such as field's @provides
    /// merged parent @provides ones. It's used to mark a QueryField as providable by a resolver
    /// simply by its existence. And while adding requirements it's used to know whether a resolver
    /// could provide it either because it's simply part of the subgraph or part of a field's
    /// @provides.
    ProvidableField(ProvidableField<'ctx, FieldId>),
}

impl<FieldId: Copy> Node<'_, FieldId> {
    /// Meant to be as readable as possible for large graphs with colors.
    pub(crate) fn label<'a, Op: Operation<FieldId = FieldId>>(&self, graph: &OperationGraph<'a, Op>) -> Cow<'a, str> {
        match self {
            Node::Root => "root".into(),
            Node::QueryField(field) => format!(
                "{}{}",
                if field.is_extra() { "*" } else { "" },
                graph.operation.field_label(field.id)
            )
            .into(),
            Node::ProvidableField(field) => match field {
                ProvidableField::InSubgraph { subgraph_id, id, .. } => format!(
                    "{}#{}",
                    graph.operation.field_label(*id),
                    subgraph_id.walk(graph.schema).name()
                ),
                ProvidableField::OnlyProvidable { subgraph_id, id, .. } => format!(
                    "{}#{}@provides",
                    graph.operation.field_label(*id),
                    subgraph_id.walk(graph.schema).name()
                ),
            }
            .into(),
            Node::Resolver(resolver) => resolver.definition_id.walk(graph.schema).name(),
        }
    }

    /// Meant to be as readable as possible for large graphs with colors.
    pub(crate) fn pretty_label<'a, Op: Operation<FieldId = FieldId>>(
        &self,
        graph: &OperationGraph<'a, Op>,
    ) -> Attrs<'a> {
        let attrs = Attrs::label(self.label(graph));
        match self {
            Node::ProvidableField(_) => attrs.with("shape=box").with("color=dodgerblue"),
            Node::Resolver(_) => attrs.with("shape=parallelogram").with("color=dodgerblue"),
            _ => attrs,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct QueryField<FieldId> {
    pub id: FieldId,
    pub flags: FieldFlags,
}

impl<FieldId> QueryField<FieldId> {
    pub fn is_indispensable(&self) -> bool {
        self.flags.contains(FieldFlags::INDISPENSABLE)
    }

    pub fn is_extra(&self) -> bool {
        self.flags.contains(FieldFlags::EXTRA)
    }

    pub fn is_typename(&self) -> bool {
        self.flags.contains(FieldFlags::TYPENAME)
    }

    pub fn is_leaf(&self) -> bool {
        self.flags.contains(FieldFlags::LEAF_NODE)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Resolver {
    pub entity_definition_id: EntityDefinitionId,
    pub definition_id: ResolverDefinitionId,
}

#[derive(Debug, Clone)]
pub(crate) enum ProvidableField<'ctx, FieldId> {
    InSubgraph {
        subgraph_id: SubgraphId,
        id: FieldId,
        provides: Cow<'ctx, FieldSetRecord>,
    },
    OnlyProvidable {
        subgraph_id: SubgraphId,
        id: FieldId,
        provides: Cow<'ctx, FieldSetRecord>,
    },
}

impl<FieldId> ProvidableField<'_, FieldId> {
    pub(crate) fn subgraph_id(&self) -> SubgraphId {
        match self {
            ProvidableField::InSubgraph { subgraph_id, .. } => *subgraph_id,
            ProvidableField::OnlyProvidable { subgraph_id, .. } => *subgraph_id,
        }
    }
}

impl<'ctx, FieldId> Node<'ctx, FieldId> {
    pub fn as_resolver(&self) -> Option<&Resolver> {
        match self {
            Node::Resolver(r) => Some(r),
            _ => None,
        }
    }

    pub fn as_providable_field(&self) -> Option<&ProvidableField<'ctx, FieldId>> {
        match self {
            Node::ProvidableField(r) => Some(r),
            _ => None,
        }
    }

    pub fn is_providable_field(&self) -> bool {
        matches!(self, Node::ProvidableField(_))
    }

    pub fn as_query_field(&self) -> Option<&QueryField<FieldId>> {
        match self {
            Node::QueryField(field) => Some(field),
            _ => None,
        }
    }
}
