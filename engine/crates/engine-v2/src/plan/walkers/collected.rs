use schema::Definition;

use crate::{
    plan::{
        AnyCollectedSelectionSet, CollectedField, CollectedFieldId, CollectedSelectionSet, CollectedSelectionSetId,
        ConditionalField, ConditionalFieldId, ConditionalSelectionSet, ConditionalSelectionSetId, FieldType,
    },
    request::SelectionSetTypeWalker,
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
        &self.operation_plan[self.item]
    }

    pub fn id(&self) -> CollectedSelectionSetId {
        self.item
    }

    pub fn ty(&self) -> SelectionSetTypeWalker<'a> {
        let ty = self.as_ref().ty;
        self.bound_walk_with(ty, Definition::from(ty))
    }

    pub fn fields(self) -> impl ExactSizeIterator<Item = PlanCollectedField<'a>> + 'a {
        self.as_ref().fields.iter().map(move |id| self.walk(id))
    }
}

impl<'a> PlanCollectedField<'a> {
    pub fn as_ref(&self) -> &'a CollectedField {
        &self.operation_plan[self.item]
    }

    pub fn as_bound_field(&self) -> PlanField<'a> {
        let field = self.as_ref();
        self.walk_with(field.bound_field_id, field.schema_field_id)
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
        &self.operation_plan[self.item]
    }

    pub fn fields(self) -> impl Iterator<Item = PlanConditionalField<'a>> + 'a {
        self.as_ref().fields.iter().map(move |id| self.walk(id))
    }
}

impl<'a> PlanConditionalField<'a> {
    pub fn as_ref(&self) -> &'a ConditionalField {
        &self.operation_plan[self.item]
    }

    pub fn as_bound_field(&self) -> PlanField<'a> {
        let field = self.as_ref();
        self.walk_with(field.bound_field_id, field.schema_field_id)
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
                        UnpackedResponseEdge::BoundResponseKey(key) => {
                            self.operation_plan.response_keys[key].to_string()
                        }
                        UnpackedResponseEdge::ExtraField(key) => self.operation_plan.response_keys[key].to_string(),
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<'a> std::fmt::Debug for PlanCollectedField<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("CollectedField");
        fmt.field("key", &self.as_bound_field().response_key_str());
        if self.as_bound_field().response_key() != self.as_ref().expected_key {
            fmt.field(
                "expected_key",
                &&self.operation_plan.response_keys[self.as_ref().expected_key],
            );
        }
        if let Some(selection_set) = self.selection_set() {
            fmt.field("selection_set", &selection_set);
        }
        fmt.finish()
    }
}

impl<'a> std::fmt::Debug for PlanConditionalSelectionSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProvisionalSelectionSet")
            .field("fields", &self.fields().collect::<Vec<_>>())
            .field(
                "typename_fields",
                &self
                    .as_ref()
                    .typename_fields
                    .iter()
                    .map(|(_, edge)| &self.operation_plan.response_keys[edge.as_response_key().unwrap()])
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<'a> std::fmt::Debug for PlanConditionalField<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("ProvisionalField");
        fmt.field("key", &self.as_bound_field().response_key_str());
        if self.as_bound_field().response_key() != self.as_ref().expected_key {
            fmt.field(
                "expected_key",
                &&self.operation_plan.response_keys[self.as_ref().expected_key],
            );
        }
        if let Some(selection_set) = self.selection_set() {
            fmt.field("selection_set", &selection_set);
        }
        fmt.finish()
    }
}
