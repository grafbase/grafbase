use walker::Walk;

use crate::{
    CostDirective, FieldDefinition, FieldSet, InputValueDefinition, ListSizeDirective, SubgraphId, TypeSystemDirective,
};

impl<'a> FieldDefinition<'a> {
    pub fn argument_by_name(&self, name: &str) -> Option<InputValueDefinition<'a>> {
        self.arguments().find(|arg| arg.name() == name)
    }

    pub fn provides_for_subgraph(&self, subgraph_id: SubgraphId) -> Option<FieldSet<'a>> {
        self.as_ref().provides_records.iter().find_map(|provide| {
            if provide.subgraph_id == subgraph_id {
                Some(provide.field_set_id.walk(self.schema))
            } else {
                None
            }
        })
    }

    pub fn requires_for_subgraph(&self, subgraph_id: SubgraphId) -> Option<FieldSet<'a>> {
        self.requires().find_map(|requires| {
            if requires.as_ref().subgraph_id == subgraph_id {
                Some(requires.field_set_id.walk(self.schema))
            } else {
                None
            }
        })
    }

    pub fn has_required_fields_for_subgraph(&self, subgraph_id: SubgraphId) -> bool {
        self.as_ref()
            .requires_records
            .iter()
            .any(|requires| requires.subgraph_id == subgraph_id)
            || self.directives().any(|directive| match directive {
                TypeSystemDirective::Authenticated
                | TypeSystemDirective::Deprecated(_)
                | TypeSystemDirective::RequiresScopes(_)
                | TypeSystemDirective::Cost(_)
                | TypeSystemDirective::Extension(_)
                | TypeSystemDirective::ListSize(_) => false,
                TypeSystemDirective::Authorized(directive) => directive.fields().is_some(),
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
}
