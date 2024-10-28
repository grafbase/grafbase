use bitflags::bitflags;
use std::borrow::Cow;

use schema::{FieldDefinitionId, ResolverDefinitionId, Schema};
use walker::Walk as _;

use super::{dot_graph::Attrs, Operation};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Node<F> {
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
    pub fn label<'a, Op: Operation<FieldId = F>>(&self, schema: &'a Schema, operation: &'a Op) -> Cow<'a, str> {
        match self {
            Node::Root => "root".into(),
            Node::QueryField(field) => format!(
                "{}{}",
                if field.is_extra() { "*" } else { "" },
                operation.field_label(field.id)
            )
            .into(),
            Node::ProvidableField(field) => format!(
                "{}@{}",
                field.field_definition_id.walk(schema).name(),
                field.resolver_definition_id.walk(schema).name()
            )
            .into(),
            Node::Resolver(resolver) => resolver.definition_id.walk(schema).name(),
        }
    }

    /// Meant to be as readable as possible for large graphs with colors.
    pub fn pretty_label<Op: Operation<FieldId = F>>(&self, schema: &Schema, operation: &Op) -> String {
        let attrs = Attrs::new(self.label(schema, operation));
        match self {
            Node::ProvidableField(_) => attrs.with("shape=box").with("style=dashed").with("color=blue"),
            Node::Resolver(_) => attrs.with("shape=box").with("color=blue"),
            _ => attrs,
        }
        .to_string()
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FieldFlags: u8 {
        /// Extra field that is not part of the operation and should not be returned to the user.
        const EXTRA = 1;
        /// Defines whether a field must be requested from the subgraphs. Operations fields are
        /// obviously indispensable, but fields necessary for @authorized also are for example.
        const INDISPENSABLE = 1 << 1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct QueryField<Id> {
    pub id: Id,
    /// Depth of the field in the query
    pub query_depth: u8,
    /// Min query depth of all the dependents of this field if any. Defaults to u8::MAX otherwise.
    /// Used to know how far away common ancestors between the dependent and the required field can
    /// be. This avoids a bunch of graph traversal during cost estimation.
    pub min_dependent_query_depth: u8,
    pub flags: FieldFlags,
}

impl<Id> QueryField<Id> {
    pub fn is_extra(&self) -> bool {
        self.flags.contains(FieldFlags::EXTRA)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Resolver {
    pub query_depth: u8,
    pub definition_id: ResolverDefinitionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ProvidableField {
    pub query_depth: u8,
    pub resolver_definition_id: ResolverDefinitionId,
    pub field_definition_id: FieldDefinitionId,
}

impl ProvidableField {
    pub fn child(&self, schema: &Schema, field_definition_id: FieldDefinitionId) -> Option<ProvidableField> {
        let resolver_definition = self.resolver_definition_id.walk(schema);
        if resolver_definition.can_provide(field_definition_id) {
            Some(ProvidableField {
                resolver_definition_id: self.resolver_definition_id,
                field_definition_id,
                query_depth: self.query_depth + 1,
            })
        } else {
            None
        }
    }
}

impl<F> Node<F> {
    pub fn query_depth(&self) -> u8 {
        match self {
            Node::Root => 0,
            Node::QueryField(field) => field.query_depth,
            Node::Resolver(resolver) => resolver.query_depth,
            Node::ProvidableField(field) => field.query_depth,
        }
    }

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

    pub fn as_query_field_mut(&mut self) -> Option<&mut QueryField<F>> {
        match self {
            Node::QueryField(field) => Some(field),
            _ => None,
        }
    }
}
