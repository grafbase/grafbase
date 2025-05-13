use std::borrow::Cow;

use operation::OperationContext;
use schema::{
    DeriveDefinitionId, EntityDefinitionId, FieldDefinitionId, FieldSetRecord, ResolverDefinitionId, SubgraphId,
};
use walker::Walk as _;

use crate::{FieldFlags, QueryFieldId, dot_graph::Attrs};

use super::QuerySolutionSpace;

#[derive(Debug, Clone)]
pub(crate) enum SpaceNode<'ctx> {
    /// Root node, unique
    Root,
    /// Field in the operation, or an extra one to satisfy requirements
    QueryField(QueryFieldNode),
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
            SpaceNode::ProvidableField(ProvidableField {
                subgraph_id,
                query_field_id,
                only_providable,
                derive: dervied_from_id,
                ..
            }) => {
                let subgraph = subgraph_id.walk(ctx).name();
                let label = crate::query::dot_graph::short_field_label(ctx, &query[*query_field_id]);

                match (only_providable, dervied_from_id) {
                    (true, None) => format!("{label}#{subgraph}@provides"),
                    (true, Some(_)) => format!("{label}#{subgraph}@provides@derive"),
                    (false, None) => format!("{label}#{subgraph}"),
                    (false, Some(_)) => format!("{label}#{subgraph}@derive"),
                }
            }
            .into(),
            SpaceNode::Resolver(resolver) => resolver.definition_id.walk(ctx).name(),
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
pub(crate) struct ProvidableField<'ctx> {
    pub subgraph_id: SubgraphId,
    pub query_field_id: QueryFieldId,
    pub provides: Cow<'ctx, FieldSetRecord>,
    pub only_providable: bool,
    pub derive: Option<Derive>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Derive {
    Root { id: DeriveDefinitionId },
    Field { from_id: FieldDefinitionId },
    ScalarAsField,
}

impl Derive {
    pub fn into_root(self) -> Option<DeriveDefinitionId> {
        match self {
            Derive::Root { id } => Some(id),
            _ => None,
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
