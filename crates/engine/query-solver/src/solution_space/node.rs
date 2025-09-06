use std::borrow::Cow;

use operation::OperationContext;
use schema::{DeriveDefinitionId, EntityDefinitionId, FieldDefinitionId, ResolverDefinitionId};
use walker::Walk as _;

use crate::{DeriveId, FieldNode, QueryFieldId, SpaceFieldSetId, dot_graph::Attrs};

use super::QuerySolutionSpace;

#[derive(Debug, Clone, Copy)]
pub(crate) enum SpaceNode {
    /// Root node, unique
    Root,
    /// Field in the operation, or an extra one to satisfy requirements
    Field(FieldNode),
    /// Defines how to access data from a subgraph
    Resolver(Resolver),
    /// Field that can be provided by a resolver with extra metadata such as field's @provides
    /// merged parent @provides ones. It's used to mark a QueryField as providable by a resolver
    /// simply by its existence. And while adding requirements it's used to know whether a resolver
    /// could provide it either because it's simply part of the subgraph or part of a field's
    /// @provides.
    ProvidableField(ProvidableField),
}

impl SpaceNode {
    /// Meant to be as readable as possible for large graphs with colors.
    pub(crate) fn label<'a>(&self, space: &QuerySolutionSpace<'_>, ctx: OperationContext<'a>) -> Cow<'a, str> {
        match self {
            SpaceNode::Root => "root".into(),
            SpaceNode::Field(node) => format!(
                "{}{}",
                if node.is_extra() { "*" } else { "" },
                crate::query::dot_graph::field_label(ctx, None, &space[node.id])
            )
            .into(),
            SpaceNode::ProvidableField(ProvidableField {
                resolver_definition_id,
                query_field_id,
                only_providable,
                derive_id,
                ..
            }) => {
                let subgraph = resolver_definition_id.walk(ctx).subgraph().name();
                let label = crate::query::dot_graph::short_field_label(ctx, &space[*query_field_id]);

                match (only_providable, derive_id) {
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
pub(crate) struct Resolver {
    pub entity_definition_id: EntityDefinitionId,
    pub definition_id: ResolverDefinitionId,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProvidableField {
    pub resolver_definition_id: ResolverDefinitionId,
    pub query_field_id: QueryFieldId,
    pub provides: Option<SpaceFieldSetId>,
    pub only_providable: bool,
    // Derive needs is quite rare so it's not worth the extra 4 bytes we would need to replace this
    // indirection.
    pub derive_id: Option<DeriveId>,
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

impl SpaceNode {
    pub fn as_resolver(&self) -> Option<&Resolver> {
        match self {
            SpaceNode::Resolver(r) => Some(r),
            _ => None,
        }
    }

    pub fn as_providable_field(&self) -> Option<&ProvidableField> {
        match self {
            SpaceNode::ProvidableField(r) => Some(r),
            _ => None,
        }
    }

    pub fn is_providable_field(&self) -> bool {
        matches!(self, SpaceNode::ProvidableField(_))
    }

    pub fn as_query_field_mut(&mut self) -> Option<&mut FieldNode> {
        match self {
            SpaceNode::Field(field) => Some(field),
            _ => None,
        }
    }

    pub fn as_query_field(&self) -> Option<&FieldNode> {
        match self {
            SpaceNode::Field(field) => Some(field),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_node_size() {
        assert_eq!(std::mem::size_of::<SpaceNode>(), 20);
        assert_eq!(std::mem::align_of::<SpaceNode>(), 4);
        assert_eq!(std::mem::size_of::<FieldNode>(), 12);
        assert_eq!(std::mem::size_of::<Resolver>(), 12);
        assert_eq!(std::mem::size_of::<ProvidableField>(), 20);
    }
}
