use std::collections::HashSet;

use schema::{FieldDefinitionWalker, TypeSystemDirective};

use crate::operation::{Condition, ConditionId, FieldId};

impl<'schema, 'p> super::Binder<'schema, 'p> {
    pub(super) fn generate_field_condition(
        &mut self,
        field_id: FieldId,
        definition: FieldDefinitionWalker<'_>,
    ) -> Option<ConditionId> {
        let mut conditions = HashSet::new();
        conditions.extend(
            definition
                .directives()
                .as_ref()
                .iter()
                .filter_map(|directive| match directive {
                    TypeSystemDirective::Authenticated => Some(self.push_condition(Condition::Authenticated)),
                    TypeSystemDirective::RequiresScopes(id) => {
                        Some(self.push_condition(Condition::RequiresScopes(*id)))
                    }
                    &TypeSystemDirective::Authorized(directive_id) => {
                        Some(self.push_condition(Condition::AuthorizedEdge { directive_id, field_id }))
                    }
                    _ => None,
                }),
        );
        conditions.extend(definition.parent_entity().directives().as_ref().iter().filter_map(
            |directive| match directive {
                &TypeSystemDirective::Authorized(directive_id) => {
                    Some(self.push_condition(Condition::AuthorizedNode {
                        directive_id,
                        entity_id: definition.parent_entity().id(),
                    }))
                }
                _ => None,
            },
        ));

        self.push_conditions(conditions)
    }

    pub(super) fn push_conditions(&mut self, conditions: HashSet<ConditionId>) -> Option<ConditionId> {
        match conditions.len() {
            0 => None,
            1 => Some(*conditions.iter().next().unwrap()),
            _ => {
                let mut conditions = conditions.into_iter().collect::<Vec<_>>();
                conditions.sort_unstable();
                Some(self.push_condition(Condition::All(conditions)))
            }
        }
    }

    fn push_condition(&mut self, condition: Condition) -> ConditionId {
        let n = self.conditions.len();
        *self.conditions.entry(condition).or_insert(n.into())
    }
}
