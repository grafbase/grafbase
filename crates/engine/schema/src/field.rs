use crate::{
    CostDirective, DeprecatedDirective, FieldDefinition, FieldSet, InputValueDefinition, ListSizeDirective, SubgraphId,
    TypeSystemDirective,
};

impl std::fmt::Display for FieldDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.parent_entity().name(), self.name())
    }
}

impl<'a> FieldDefinition<'a> {
    pub fn argument_by_name(&self, name: &str) -> Option<InputValueDefinition<'a>> {
        self.arguments().find(|arg| arg.name() == name)
    }

    pub fn provides_for_subgraph(&self, subgraph_id: SubgraphId) -> Option<FieldSet<'a>> {
        self.provides().find_map(|provide| {
            if provide.subgraph_id == subgraph_id {
                Some(provide.field_set())
            } else {
                None
            }
        })
    }

    pub fn requires_for_subgraph(&self, subgraph_id: SubgraphId) -> Option<FieldSet<'a>> {
        self.requires().find_map(|requires| {
            if requires.as_ref().subgraph_id == subgraph_id {
                Some(requires.field_set())
            } else {
                None
            }
        })
    }

    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_field_definitions[self.id]
    }

    pub fn cost(&self) -> Option<CostDirective> {
        self.directives().find_map(|directive| match directive {
            TypeSystemDirective::Cost(cost) => Some(cost),
            _ => None,
        })
    }

    pub fn list_size(&self) -> Option<ListSizeDirective<'a>> {
        self.directives().find_map(|directive| match directive {
            TypeSystemDirective::ListSize(list_size_directive) => Some(list_size_directive),
            _ => None,
        })
    }

    pub fn has_deprecated(&self) -> Option<DeprecatedDirective<'_>> {
        self.directives().find_map(|directive| directive.as_deprecated())
    }
}

impl std::fmt::Debug for FieldDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldDefinition")
            .field("name", &self.name())
            .field("parent_entity", &self.parent_entity().name())
            .field("ty", &self.ty())
            .field("arguments", &self.arguments())
            .finish()
    }
}
