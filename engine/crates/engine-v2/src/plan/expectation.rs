use schema::{DataType, FieldId, Wrapping};

use super::{ExtraFieldId, ExtraSelectionSet, PlanBoundaryId};

use crate::{
    request::{BoundAnyFieldDefinitionId, FlatTypeCondition, SelectionSetType},
    response::{BoundResponseKey, ResponseEdge},
};

mod ids {
    use super::*;

    crate::utils::id_newtypes! {
        ExpectationsBuilder.fields[ExpectedFieldId] => ExpectedField unless "Too many ungrouped fields",
        ExpectationsBuilder.undetermined_selection_sets[UndeterminedSelectionSetId] => UndeterminedSelectionSet unless "Too many ungrouped selection sets",
    }
}

pub use ids::*;

#[derive(Default, Debug)]
pub struct ExpectationsBuilder {
    undetermined_selection_sets: Vec<UndeterminedSelectionSet>,
    fields: Vec<ExpectedField>,
}

impl ExpectationsBuilder {
    pub(super) fn push_field(&mut self, field: ExpectedField) -> ExpectedFieldId {
        let id = ExpectedFieldId::from(self.fields.len());
        self.fields.push(field);
        id
    }

    pub(super) fn push_ungrouped_selection_set(
        &mut self,
        ungrouped_selection_set: UndeterminedSelectionSet,
    ) -> UndeterminedSelectionSetId {
        let id = UndeterminedSelectionSetId::from(self.undetermined_selection_sets.len());
        self.undetermined_selection_sets.push(ungrouped_selection_set);
        id
    }

    pub(super) fn build(self, root_selection_set: CollectedSelectionSet) -> Expectations {
        Expectations {
            root_selection_set,
            undetermined_selection_sets: self.undetermined_selection_sets,
            fields: self.fields,
        }
    }
}

#[derive(Debug)]
pub struct Expectations {
    pub root_selection_set: CollectedSelectionSet,
    undetermined_selection_sets: Vec<UndeterminedSelectionSet>,
    fields: Vec<ExpectedField>,
}

impl std::ops::Index<ExpectedFieldId> for Expectations {
    type Output = ExpectedField;

    fn index(&self, index: ExpectedFieldId) -> &Self::Output {
        &self.fields[usize::from(index)]
    }
}

impl std::ops::Index<UndeterminedSelectionSetId> for Expectations {
    type Output = UndeterminedSelectionSet;

    fn index(&self, index: UndeterminedSelectionSetId) -> &Self::Output {
        &self.undetermined_selection_sets[usize::from(index)]
    }
}

#[derive(Debug)]
pub enum ExpectedSelectionSet {
    Collected(CollectedSelectionSet),
    Undetermined(UndeterminedSelectionSetId),
    MergedUndetermined {
        ty: SelectionSetType,
        selection_set_ids: Vec<UndeterminedSelectionSetId>,
    },
}

#[derive(Debug, Clone)]
pub struct UndeterminedSelectionSet {
    // needed to know where to look for __typename
    pub ty: SelectionSetType,
    pub maybe_boundary_id: Option<PlanBoundaryId>,
    // sorted by ResponseEdge, so bound response key and then extra fields
    pub fields: Vec<PossibleField>,
}

#[derive(Debug, Clone)]
pub enum PossibleField {
    TypeName {
        type_condition: Option<FlatTypeCondition>,
        key: BoundResponseKey,
    },
    Query(ExpectedFieldId),
    Extra(ExtraFieldId),
}

#[derive(Debug)]
pub struct ExpectedField {
    pub type_condition: Option<FlatTypeCondition>,
    pub bound_response_key: BoundResponseKey,
    pub expected_key: String,
    pub field_id: FieldId,
    pub definition_id: BoundAnyFieldDefinitionId,
    pub ty: ExpectedType<UndeterminedSelectionSetId>,
}

#[derive(Debug, Clone)]
pub enum ExpectedType<Id> {
    Scalar(DataType),
    SelectionSet(Id),
}

#[derive(Debug)]
pub struct CollectedSelectionSet {
    pub ty: SelectionSetType,
    pub boundary_ids: Vec<PlanBoundaryId>,
    // sorted by expected key
    pub fields: Vec<ConcreteField>,
    pub typename_fields: Vec<ResponseEdge>,
}

#[derive(Debug)]
pub struct ConcreteField {
    pub edge: ResponseEdge,
    pub expected_key: String,
    pub definition_id: Option<BoundAnyFieldDefinitionId>,
    pub ty: ConcreteType,
    pub wrapping: Wrapping,
}

#[derive(Debug)]
pub enum ConcreteType {
    Scalar(DataType),
    SelectionSet(ExpectedSelectionSet),
    ExtraSelectionSet(ExtraSelectionSet),
}

impl std::fmt::Display for ConcreteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConcreteType::Scalar(data_type) => write!(f, "{data_type}"),
            ConcreteType::SelectionSet(_) => write!(f, "Object"),
            ConcreteType::ExtraSelectionSet(_) => write!(f, "Extra"),
        }
    }
}
