use std::collections::HashSet;

use schema::{EntityWalker, FieldDefinitionWalker, TypeSystemDirective};

use crate::operation::{Condition, ConditionId, FieldId};

impl<'schema, 'p> super::Binder<'schema, 'p> {
    pub(super) fn generate_field_condition(
        &mut self,
        field_id: FieldId,
        definition: FieldDefinitionWalker<'_>,
    ) -> Option<ConditionId> {
        let mut conditions: HashSet<_> = definition
            .directives()
            .as_ref()
            .iter()
            .filter_map(|directive| match directive {
                TypeSystemDirective::Authenticated => Some(self.push_condition(Condition::Authenticated)),
                TypeSystemDirective::RequiresScopes(id) => Some(self.push_condition(Condition::RequiresScopes(*id))),
                &TypeSystemDirective::Authorized(directive_id) => {
                    Some(self.push_condition(Condition::AuthorizedEdge { directive_id, field_id }))
                }
                _ => None,
            })
            .collect();

        // FIXME: doesn't take into account objects behind interfaces/unions
        if let Some(entity) = definition.ty().inner().as_entity() {
            conditions.extend(self.generate_entity_conditions(entity));
        }

        self.push_conditions(conditions)
    }

    pub(super) fn generate_entity_conditions(&mut self, entity: EntityWalker<'_>) -> HashSet<ConditionId> {
        entity
            .directives()
            .as_ref()
            .iter()
            .filter_map(|directive| match directive {
                &TypeSystemDirective::Authorized(directive_id) => {
                    Some(self.push_condition(Condition::AuthorizedNode {
                        directive_id,
                        entity_id: entity.id(),
                    }))
                }
                _ => None,
            })
            .collect()
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
