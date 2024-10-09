use std::borrow::Cow;

use schema::{FieldDefinition, FieldDefinitionId, ResolverDefinitionId, Schema};
use walker::Walk as _;

use super::{dot_graph::Attrs, Operation};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Node<F> {
    /// Root node, unique
    Root,
    /// Field in the operation, or an extra one to satisfy requirements
    Field(F),
    /// Defines how to access data from a subgraph
    Resolver(Resolver),
    /// Field resolver by a resolver with extra metadata such as parent's @provides.
    FieldResolver(FieldResolver),
}

impl<F: Copy> Node<F> {
    /// Meant to be as readable as possible for large graphs with colors.
    pub(crate) fn label<'a, Op: Operation<FieldId = F>>(&self, schema: &'a Schema, operation: &'a Op) -> Cow<'a, str> {
        match self {
            Node::Root => "root".into(),
            Node::Field(id) => operation.field_label(*id),
            Node::FieldResolver(field_resolver) => format!(
                "{}@{}",
                field_resolver.field_definition_id.walk(schema).name(),
                field_resolver.resolver_definition_id.walk(schema).name()
            )
            .into(),
            Node::Resolver(resolver) => resolver.definition_id.walk(schema).name(),
        }
    }

    /// Meant to be as readable as possible for large graphs with colors.
    pub(crate) fn pretty_label<Op: Operation<FieldId = F>>(&self, schema: &Schema, operation: &Op) -> String {
        let attrs = Attrs::new(self.label(schema, operation));
        match self {
            Node::FieldResolver(_) => attrs.with("shape=box").with("style=dashed").with("color=blue"),
            Node::Resolver(_) => attrs.with("shape=box").with("color=blue"),
            _ => attrs,
        }
        .to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Resolver {
    pub(crate) definition_id: ResolverDefinitionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FieldResolver {
    pub(crate) resolver_definition_id: ResolverDefinitionId,
    pub(crate) field_definition_id: FieldDefinitionId,
}

impl FieldResolver {
    pub(crate) fn new(resolver_definition_id: ResolverDefinitionId, field_definition: FieldDefinition<'_>) -> Self {
        FieldResolver {
            resolver_definition_id,
            field_definition_id: field_definition.id(),
        }
    }

    pub(crate) fn child(&self, schema: &Schema, field_definition_id: FieldDefinitionId) -> Option<FieldResolver> {
        let resolver_definition = self.resolver_definition_id.walk(schema);
        if resolver_definition.can_provide(field_definition_id) {
            Some(FieldResolver {
                resolver_definition_id: self.resolver_definition_id,
                field_definition_id,
            })
        } else {
            None
        }
    }
}

impl<F> Node<F> {
    pub(crate) fn as_resolver(&self) -> Option<&Resolver> {
        match self {
            Node::Resolver(r) => Some(r),
            _ => None,
        }
    }

    pub(crate) fn as_field_resolver(&self) -> Option<&FieldResolver> {
        match self {
            Node::FieldResolver(r) => Some(r),
            _ => None,
        }
    }

    pub(crate) fn as_field(&self) -> Option<F>
    where
        F: Copy,
    {
        match self {
            Node::Field(field_id) => Some(*field_id),
            _ => None,
        }
    }
}
