use std::borrow::Cow;

use walker::Walk;

use crate::{
    FieldDefinition, InputValueDefinition, ProvidableFieldSet, RequiredFieldSet, RequiredFieldSetRecord, SubgraphId,
    TypeSystemDirective,
};

impl<'a> FieldDefinition<'a> {
    pub fn argument_by_name(&self, name: &str) -> Option<InputValueDefinition<'a>> {
        self.arguments().find(|arg| arg.name() == name)
    }

    pub fn provides_for_subgraph(&self, subgraph_id: SubgraphId) -> &'a ProvidableFieldSet {
        self.as_ref()
            .provides_records
            .iter()
            .find_map(|provide| {
                if provide.subgraph_id == subgraph_id {
                    Some(&provide.field_set)
                } else {
                    None
                }
            })
            .unwrap_or(ProvidableFieldSet::empty())
    }

    pub fn requires_for_subgraph(&self, subgraph_id: SubgraphId) -> Option<RequiredFieldSet<'a>> {
        self.requires().find_map(|requires| {
            if requires.as_ref().subgraph_id == subgraph_id {
                Some(requires.field_set_id.walk(self.schema))
            } else {
                None
            }
        })
    }

    pub fn all_requires_for_subgraph(&self, subgraph_id: SubgraphId) -> Cow<'a, RequiredFieldSetRecord> {
        self.directives()
            .filter_map(|directive| match directive {
                TypeSystemDirective::Authenticated
                | TypeSystemDirective::Deprecated(_)
                | TypeSystemDirective::RequiresScopes(_) => None,
                TypeSystemDirective::Authorized(directive) => {
                    directive.fields().map(|fields| Cow::Borrowed(fields.as_ref()))
                }
            })
            .chain(self.requires().find_map(|requires| {
                if requires.as_ref().subgraph_id == subgraph_id {
                    Some(Cow::Borrowed(requires.field_set().as_ref()))
                } else {
                    None
                }
            }))
            .reduce(RequiredFieldSetRecord::union_cow)
            .unwrap_or(Cow::Borrowed(RequiredFieldSetRecord::empty()))
    }

    pub fn is_resolvable_in(&self, subgraph_id: SubgraphId) -> bool {
        self.only_resolvable_in_ids.is_empty() || self.only_resolvable_in_ids.contains(&subgraph_id)
    }

    pub fn has_required_fields_for_subgraph(&self, subgraph_id: SubgraphId) -> bool {
        self.as_ref()
            .requires_records
            .iter()
            .any(|requires| requires.subgraph_id == subgraph_id)
            || self.directives().any(|directive| match directive {
                TypeSystemDirective::Authenticated
                | TypeSystemDirective::Deprecated(_)
                | TypeSystemDirective::RequiresScopes(_) => false,
                TypeSystemDirective::Authorized(directive) => directive.fields().is_some(),
            })
    }
}
