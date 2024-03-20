use schema::Definition;

use crate::{
    operation::SelectionSetTypeWalker,
    plan::{
        AnyCollectedSelectionSet, CollectedField, CollectedFieldId, CollectedSelectionSet, CollectedSelectionSetId,
        ConditionalField, ConditionalFieldId, ConditionalSelectionSet, ConditionalSelectionSetId, FieldType,
    },
    response::UnpackedResponseEdge,
};

use super::{PlanField, PlanWalker};

pub type PlanCollectedSelectionSet<'a> = PlanWalker<'a, CollectedSelectionSetId, ()>;
pub type PlanCollectedField<'a> = PlanWalker<'a, CollectedFieldId, ()>;
pub type PlanConditionalSelectionSet<'a> = PlanWalker<'a, ConditionalSelectionSetId, ()>;
pub type PlanConditionalField<'a> = PlanWalker<'a, ConditionalFieldId, ()>;

pub enum PlanAnyCollectedSelectionSet<'a> {
    Collected(PlanCollectedSelectionSet<'a>),
    Conditional(PlanConditionalSelectionSet<'a>),
}

impl<'a> PlanCollectedSelectionSet<'a> {
    pub fn as_ref(&self) -> &'a CollectedSelectionSet {
        &self.operation_plan[*self._item()]
    }

    pub fn id(&self) -> CollectedSelectionSetId {
        *self._item()
    }

    pub fn ty(&self) -> SelectionSetTypeWalker<'a> {
        let ty = self.as_ref().ty;
        self.bound_walk_with(ty, Definition::from(ty))
    }

    pub fn fields(self) -> impl ExactSizeIterator<Item = PlanCollectedField<'a>> + 'a {
        self.as_ref().field_ids.map(move |id| self.walk(id))
    }
}

impl<'a> PlanCollectedField<'a> {
    pub fn as_ref(&self) -> &'a CollectedField {
        &self.operation_plan[*self._item()]
    }

    pub fn as_operation_field(&self) -> PlanField<'a> {
        let field = self.as_ref();
        self.walk_with(field.id, field.definition_id)
    }

    pub fn concrete_selection_set(&self) -> Option<PlanCollectedSelectionSet<'a>> {
        if let FieldType::SelectionSet(AnyCollectedSelectionSet::Collected(id)) = self.as_ref().ty {
            Some(self.walk(id))
        } else {
            None
        }
    }

    pub fn selection_set(&self) -> Option<PlanAnyCollectedSelectionSet<'a>> {
        match self.as_ref().ty {
            FieldType::SelectionSet(AnyCollectedSelectionSet::Collected(id)) => {
                Some(PlanAnyCollectedSelectionSet::Collected(self.walk(id)))
            }
            FieldType::SelectionSet(AnyCollectedSelectionSet::Conditional(id)) => {
                Some(PlanAnyCollectedSelectionSet::Conditional(self.walk(id)))
            }
            _ => None,
        }
    }
}

impl<'a> PlanConditionalSelectionSet<'a> {
    pub fn as_ref(&self) -> &'a ConditionalSelectionSet {
        &self.operation_plan[*self._item()]
    }

    pub fn fields(self) -> impl Iterator<Item = PlanConditionalField<'a>> + 'a {
        self.as_ref().field_ids.map(move |id| self.walk(id))
    }
}

impl<'a> PlanConditionalField<'a> {
    pub fn as_ref(&self) -> &'a ConditionalField {
        &self.operation_plan[*self._item()]
    }

    pub fn as_operation_field(&self) -> PlanField<'a> {
        let field = self.as_ref();
        self.walk_with(field.id, field.definition_id)
    }

    pub fn selection_set(&self) -> Option<PlanConditionalSelectionSet<'a>> {
        match self.as_ref().ty {
            FieldType::SelectionSet(id) => Some(self.walk(id)),
            _ => None,
        }
    }
}

impl<'a> std::fmt::Debug for PlanAnyCollectedSelectionSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanAnyCollectedSelectionSet::Collected(selection_set) => selection_set.fmt(f),
            PlanAnyCollectedSelectionSet::Conditional(selection_set) => selection_set.fmt(f),
        }
    }
}

impl<'a> std::fmt::Debug for PlanCollectedSelectionSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let response_keys = &self.walk(())._operation().response_keys;
        f.debug_struct("CollectedSelectionSet")
            .field("fields", &self.fields().collect::<Vec<_>>())
            .field(
                "typename_fields",
                &self
                    .as_ref()
                    .typename_fields
                    .iter()
                    .map(|edge| match edge.unpack() {
                        UnpackedResponseEdge::Index(i) => format!("index: {i}"),
                        UnpackedResponseEdge::BoundResponseKey(key) => response_keys[key].to_string(),
                        UnpackedResponseEdge::ExtraFieldResponseKey(key) => response_keys[key].to_string(),
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<'a> std::fmt::Debug for PlanCollectedField<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let response_keys = &self.walk(())._operation().response_keys;
        let mut fmt = f.debug_struct("CollectedField");
        fmt.field("key", &self.as_operation_field().response_key_str());
        if self.as_operation_field().response_key() != self.as_ref().expected_key.into() {
            fmt.field("expected_key", &&response_keys[self.as_ref().expected_key]);
        }
        if let Some(selection_set) = self.selection_set() {
            fmt.field("selection_set", &selection_set);
        }
        fmt.finish()
    }
}

impl<'a> std::fmt::Debug for PlanConditionalSelectionSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let response_keys = &self.walk(())._operation().response_keys;
        f.debug_struct("ProvisionalSelectionSet")
            .field("fields", &self.fields().collect::<Vec<_>>())
            .field(
                "typename_fields",
                &self
                    .as_ref()
                    .typename_fields
                    .iter()
                    .map(|(_, edge)| &response_keys[edge.as_response_key().unwrap()])
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<'a> std::fmt::Debug for PlanConditionalField<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let response_keys = &self.walk(())._operation().response_keys;
        let mut fmt = f.debug_struct("ProvisionalField");
        fmt.field("key", &self.as_operation_field().response_key_str());
        if self.as_operation_field().response_key() != self.as_ref().expected_key.into() {
            fmt.field("expected_key", &&response_keys[self.as_ref().expected_key]);
        }
        if let Some(selection_set) = self.selection_set() {
            fmt.field("selection_set", &selection_set);
        }
        fmt.finish()
    }
}
