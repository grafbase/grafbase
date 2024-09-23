use petgraph::graph::NodeIndex;
use schema::{FieldDefinition, FieldDefinitionId, ResolverDefinitionId, Schema};
use walker::Walk as _;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node<F> {
    Root,
    Field(F),
    Resolver(Resolver),
    FieldResolver(FieldResolver),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Resolver {
    pub parent_resolver_node: NodeIndex,
    pub definition_id: ResolverDefinitionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldResolver {
    pub parent_resolver_node: NodeIndex,
    pub resolver_definition_id: ResolverDefinitionId,
    pub field_definition_id: FieldDefinitionId,
}

impl FieldResolver {
    pub fn new(
        parent_resolver_node: NodeIndex,
        resolver_definition_id: ResolverDefinitionId,
        field_definition: FieldDefinition<'_>,
    ) -> Self {
        FieldResolver {
            parent_resolver_node,
            resolver_definition_id,
            field_definition_id: field_definition.id(),
        }
    }

    pub fn child(&self, schema: &Schema, field_definition_id: FieldDefinitionId) -> Option<FieldResolver> {
        let resolver_definition = self.resolver_definition_id.walk(schema);
        if resolver_definition.can_provide(field_definition_id) {
            Some(FieldResolver {
                parent_resolver_node: self.parent_resolver_node,
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

    pub fn as_field_resolver(&self) -> Option<&FieldResolver> {
        match self {
            Node::FieldResolver(r) => Some(r),
            _ => None,
        }
    }

    pub fn as_field(&self) -> Option<F>
    where
        F: Copy,
    {
        match self {
            Node::Field(field_id) => Some(*field_id),
            _ => None,
        }
    }
}
