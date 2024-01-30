use schema::Definition;

use crate::{
    plan::{
        CollectedSelectionSet, ConcreteField, ConcreteFieldId, ConcreteSelectionSet, ConcreteSelectionSetId,
        ConditionalField, ConditionalFieldId, ConditionalSelectionSet, ConditionalSelectionSetId, FieldType,
    },
    request::SelectionSetTypeWalker,
    response::UnpackedResponseEdge,
};

use super::{PlanField, PlanWalker};

pub type PlanConcreteSelectionSet<'a> = PlanWalker<'a, ConcreteSelectionSetId, ()>;
pub type PlanConcreteField<'a> = PlanWalker<'a, ConcreteFieldId, ()>;
pub type PlanConditionalSelectionSet<'a> = PlanWalker<'a, ConditionalSelectionSetId, ()>;
pub type PlanConditionalField<'a> = PlanWalker<'a, ConditionalFieldId, ()>;

pub enum PlanCollectedSelectionSet<'a> {
    Concrete(PlanConcreteSelectionSet<'a>),
    Provisional(PlanConditionalSelectionSet<'a>),
}

impl<'a> PlanConcreteSelectionSet<'a> {
    pub fn as_ref(&self) -> &'a ConcreteSelectionSet {
        &self.operation[self.item]
    }

    pub fn id(&self) -> ConcreteSelectionSetId {
        self.item
    }

    pub fn ty(&self) -> SelectionSetTypeWalker<'a> {
        let ty = self.as_ref().ty;
        self.bound_walk_with(ty, Definition::from(ty))
    }

    pub fn fields(self) -> impl ExactSizeIterator<Item = PlanConcreteField<'a>> + 'a {
        self.as_ref().fields.iter().map(move |id| self.walk(id))
    }
}

impl<'a> PlanConcreteField<'a> {
    pub fn as_ref(&self) -> &'a ConcreteField {
        &self.operation[self.item]
    }

    pub fn as_bound_field(&self) -> PlanField<'a> {
        let field = self.as_ref();
        self.walk_with(field.bound_field_id, field.schema_field_id)
    }

    pub fn concrete_selection_set(&self) -> Option<PlanConcreteSelectionSet<'a>> {
        if let FieldType::SelectionSet(CollectedSelectionSet::Concrete(id)) = self.as_ref().ty {
            Some(self.walk(id))
        } else {
            None
        }
    }

    pub fn selection_set(&self) -> Option<PlanCollectedSelectionSet<'a>> {
        match self.as_ref().ty {
            FieldType::SelectionSet(CollectedSelectionSet::Concrete(id)) => {
                Some(PlanCollectedSelectionSet::Concrete(self.walk(id)))
            }
            FieldType::SelectionSet(CollectedSelectionSet::Conditional(id)) => {
                Some(PlanCollectedSelectionSet::Provisional(self.walk(id)))
            }
            _ => None,
        }
    }
}

impl<'a> PlanConditionalSelectionSet<'a> {
    pub fn as_ref(&self) -> &'a ConditionalSelectionSet {
        &self.operation[self.item]
    }

    pub fn fields(self) -> impl Iterator<Item = PlanConditionalField<'a>> + 'a {
        self.as_ref().fields.iter().map(move |id| self.walk(id))
    }
}

impl<'a> PlanConditionalField<'a> {
    pub fn as_ref(&self) -> &'a ConditionalField {
        &self.operation[self.item]
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

impl<'a> std::fmt::Debug for PlanCollectedSelectionSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanCollectedSelectionSet::Concrete(selection_set) => selection_set.fmt(f),
            PlanCollectedSelectionSet::Provisional(selection_set) => selection_set.fmt(f),
        }
    }
}

impl<'a> std::fmt::Debug for PlanConcreteSelectionSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConcreteSelectionSet")
            .field("fields", &self.fields().collect::<Vec<_>>())
            .field(
                "typename_fields",
                &self
                    .as_ref()
                    .typename_fields
                    .iter()
                    .map(|edge| match edge.unpack() {
                        UnpackedResponseEdge::Index(i) => format!("index: {i}"),
                        UnpackedResponseEdge::BoundResponseKey(key) => self.operation.response_keys[key].to_string(),
                        UnpackedResponseEdge::ExtraField(key) => self.operation.response_keys[key].to_string(),
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl<'a> std::fmt::Debug for PlanConcreteField<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("ConcreteField");
        fmt.field("key", &self.as_bound_field().response_key_str());
        if self.as_bound_field().response_key() != self.as_ref().expected_key {
            fmt.field(
                "expected_key",
                &&self.operation.response_keys[self.as_ref().expected_key],
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
                    .map(|(_, edge)| &self.operation.response_keys[edge.as_response_key().unwrap()])
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
                &&self.operation.response_keys[self.as_ref().expected_key],
            );
        }
        if let Some(selection_set) = self.selection_set() {
            fmt.field("selection_set", &selection_set);
        }
        fmt.finish()
    }
}
