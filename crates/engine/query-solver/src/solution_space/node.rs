use std::borrow::Cow;

use operation::OperationContext;
use schema::{EntityDefinitionId, FieldSetRecord, ResolverDefinitionId, SubgraphId};
use walker::Walk as _;

use crate::{dot_graph::Attrs, FieldFlags, QueryFieldId};

use super::QuerySolutionSpace;

#[derive(Debug, Clone)]
pub(crate) enum SpaceNode<'ctx> {
    /// Root node, unique
    Root,
    /// Field in the operation, or an extra one to satisfy requirements
    QueryField(QueryFieldNode),
    Typename(TypenameFieldNode),
    /// Defines how to access data from a subgraph
    Resolver(Resolver),
    /// Field that can be provided by a resolver with extra metadata such as field's @provides
    /// merged parent @provides ones. It's used to mark a QueryField as providable by a resolver
    /// simply by its existence. And while adding requirements it's used to know whether a resolver
    /// could provide it either because it's simply part of the subgraph or part of a field's
    /// @provides.
    ProvidableField(ProvidableField<'ctx>),
}

impl SpaceNode<'_> {
    /// Meant to be as readable as possible for large graphs with colors.
    pub(crate) fn label<'a>(&self, query: &QuerySolutionSpace<'_>, ctx: OperationContext<'a>) -> Cow<'a, str> {
        match self {
            SpaceNode::Root => "root".into(),
            SpaceNode::QueryField(node) => format!(
                "{}{}",
                if node.is_extra() { "*" } else { "" },
                crate::query::dot_graph::field_label(ctx, &query[node.id])
            )
            .into(),
            SpaceNode::ProvidableField(node) => match node {
                ProvidableField::InSubgraph {
                    subgraph_id, field_id, ..
                } => format!(
                    "{}#{}",
                    crate::query::dot_graph::short_field_label(ctx, &query[*field_id]),
                    subgraph_id.walk(ctx).name()
                ),
                ProvidableField::OnlyProvidable {
                    subgraph_id, field_id, ..
                } => format!(
                    "{}#{}@provides",
                    crate::query::dot_graph::short_field_label(ctx, &query[*field_id]),
                    subgraph_id.walk(ctx).name()
                ),
            }
            .into(),
            SpaceNode::Resolver(resolver) => resolver.definition_id.walk(ctx).name(),
            SpaceNode::Typename(_) => "__typename".into(),
        }
    }

    /// Meant to be as readable as possible for large graphs with colors.
    pub(crate) fn pretty_label<'a>(&self, query: &QuerySolutionSpace<'_>, ctx: OperationContext<'a>) -> Attrs<'a> {
        let attrs = Attrs::label(self.label(query, ctx));
        match self {
            SpaceNode::ProvidableField(_) => attrs.with("shape=box").with("color=dodgerblue"),
            SpaceNode::Resolver(_) => attrs.with("shape=parallelogram").with("color=dodgerblue"),
            _ => attrs,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct QueryFieldNode {
    pub id: QueryFieldId,
    pub flags: FieldFlags,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TypenameFieldNode {
    pub indispensable: bool,
}

impl QueryFieldNode {
    pub fn is_indispensable(&self) -> bool {
        self.flags.contains(FieldFlags::INDISPENSABLE)
    }

    pub fn is_extra(&self) -> bool {
        self.flags.contains(FieldFlags::EXTRA)
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
pub(crate) enum ProvidableField<'ctx> {
    InSubgraph {
        subgraph_id: SubgraphId,
        field_id: QueryFieldId,
        provides: Cow<'ctx, FieldSetRecord>,
    },
    OnlyProvidable {
        subgraph_id: SubgraphId,
        field_id: QueryFieldId,
        provides: Cow<'ctx, FieldSetRecord>,
    },
}

impl ProvidableField<'_> {
    pub(crate) fn subgraph_id(&self) -> SubgraphId {
        match self {
            ProvidableField::InSubgraph { subgraph_id, .. } => *subgraph_id,
            ProvidableField::OnlyProvidable { subgraph_id, .. } => *subgraph_id,
        }
    }
}

impl<'ctx> SpaceNode<'ctx> {
    pub fn as_resolver(&self) -> Option<&Resolver> {
        match self {
            SpaceNode::Resolver(r) => Some(r),
            _ => None,
        }
    }

    pub fn as_providable_field(&self) -> Option<&ProvidableField<'ctx>> {
        match self {
            SpaceNode::ProvidableField(r) => Some(r),
            _ => None,
        }
    }

    pub fn is_providable_field(&self) -> bool {
        matches!(self, SpaceNode::ProvidableField(_))
    }

    pub fn as_query_field_mut(&mut self) -> Option<&mut QueryFieldNode> {
        match self {
            SpaceNode::QueryField(field) => Some(field),
            _ => None,
        }
    }

    pub fn as_query_field(&self) -> Option<&QueryFieldNode> {
        match self {
            SpaceNode::QueryField(field) => Some(field),
            _ => None,
        }
    }
}
