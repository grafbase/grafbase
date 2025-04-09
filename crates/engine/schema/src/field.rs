use crate::{
    CostDirective, DeprecatedDirective, FieldDefinition, FieldSet, InputValueDefinition, ListSizeDirective, SubgraphId,
    TypeSystemDirective,
};

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
