use super::{resolver::ResolverWalker, SchemaWalker};
use crate::{
    EntityId, FieldDefinitionId, InputValueDefinitionWalker, ProvidableFieldSet, RequiredFieldSet, SubgraphId,
    TypeSystemDirectivesWalker, TypeWalker,
};

pub type FieldDefinitionWalker<'a> = SchemaWalker<'a, FieldDefinitionId>;

impl<'a> FieldDefinitionWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.field(self.schema, self.item)
    }

    pub fn resolvers(self) -> impl ExactSizeIterator<Item = ResolverWalker<'a>> {
        self.schema[self.item].resolvers.iter().map(move |id| self.walk(*id))
    }

    pub fn is_resolvable_in(&self, subgraph_id: SubgraphId) -> bool {
        let r = &self.as_ref().only_resolvable_in;
        r.is_empty() || r.contains(&subgraph_id)
    }

    pub fn provides(&self, subgraph_id: SubgraphId) -> &'a ProvidableFieldSet {
        self.as_ref()
            .provides
            .iter()
            .find_map(|provide| {
                if provide.subgraph_id == subgraph_id {
                    Some(&provide.field_set)
                } else {
                    None
                }
            })
            .unwrap_or(&crate::provides::EMPTY)
    }

    pub fn requires(&self, subgraph_id: SubgraphId) -> &'a RequiredFieldSet {
        self.as_ref()
            .requires
            .iter()
            .find_map(|requires| {
                if requires.subgraph_id == subgraph_id {
                    Some(&self.schema[requires.field_set_id])
                } else {
                    None
                }
            })
            .unwrap_or(&crate::requires::EMPTY)
    }

    pub fn parent_entity(&self) -> EntityId {
        self.as_ref().parent_entity
    }

    pub fn arguments(self) -> impl ExactSizeIterator<Item = InputValueDefinitionWalker<'a>> + 'a {
        self.schema[self.item].argument_ids.map(move |id| self.walk(id))
    }

    pub fn ty(self) -> TypeWalker<'a> {
        self.walk(self.as_ref().ty)
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directives)
    }

    pub fn argument_by_name(&self, name: &str) -> Option<InputValueDefinitionWalker<'a>> {
        self.arguments().find(|arg| arg.name() == name)
    }
}

pub struct FieldResolverWalker<'a> {
    pub resolver: ResolverWalker<'a>,
    pub field_requires: &'a RequiredFieldSet,
}

impl<'a> std::fmt::Debug for FieldDefinitionWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field("type", &self.ty().to_string())
            .field("resolvable_in", &self.as_ref().only_resolvable_in)
            .field("resolvers", &self.resolvers().collect::<Vec<_>>())
            .field(
                "arguments",
                &self
                    .arguments()
                    .map(|arg| (arg.name(), arg.ty().to_string()))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<'a> std::fmt::Debug for FieldResolverWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldResolver")
            .field("resolver", &self.resolver)
            .field("requires", &self.resolver.walk(self.field_requires))
            .finish()
    }
}
