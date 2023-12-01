use itertools::Itertools;
use schema::{DataType, Wrapping};

use crate::{
    request::{BoundAnyFieldDefinitionId, BoundFieldId, BoundSelectionSetId, FlatTypeCondition, SelectionSetRoot},
    response::BoundResponseKey,
};

#[derive(Debug)]
pub enum ExpectedSelectionSet {
    Grouped(ExpectedGroupedFields),
    Arbitrary(ExpectedArbitraryFields),
}

#[derive(Debug)]
pub struct ExpectedGroupedFields {
    pub root: SelectionSetRoot,
    // sorted by expected name
    pub fields: Vec<ExpectedGoupedField>,
    pub typename_fields: Vec<BoundResponseKey>,
}

pub enum FieldOrTypeName {
    Field(ExpectedGoupedField),
    TypeName(BoundResponseKey),
}

impl ExpectedGroupedFields {
    pub fn new(root: SelectionSetRoot, fields: impl IntoIterator<Item = FieldOrTypeName>) -> Self {
        let (mut fields, typename_fields): (Vec<_>, Vec<_>) = fields
            .into_iter()
            .map(|field| match field {
                FieldOrTypeName::Field(field) => Ok(field),
                FieldOrTypeName::TypeName(key) => Err(key),
            })
            .partition_result();
        fields.sort_unstable_by_key(|field| field.expected_name.clone());
        Self {
            root,
            fields,
            typename_fields,
        }
    }
}

#[derive(Debug)]
pub struct ExpectedGoupedField {
    pub bound_response_key: BoundResponseKey,
    pub expected_name: String,
    pub definition_id: BoundAnyFieldDefinitionId,
    pub ty: ExpectedType,
    pub wrapping: Wrapping,
}

#[derive(Debug, Clone)]
pub struct ExpectedUngroupedField {
    pub expected_name: Option<String>,
    pub type_condition: Option<FlatTypeCondition>,
    pub origin: BoundSelectionSetId,
    pub bound_field_id: BoundFieldId,
    pub ty: ExpectedType<ExpectedArbitraryFields>,
}

#[derive(Debug, Clone)]
pub struct ExpectedArbitraryFields {
    // needed to know where to look for __typename
    pub root: SelectionSetRoot,
    pub fields: Vec<ExpectedUngroupedField>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExpectedType<SelectionSet = ExpectedSelectionSet> {
    TypeName,
    Scalar(DataType),
    Object(Box<SelectionSet>),
}

impl<T> std::fmt::Display for ExpectedType<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpectedType::TypeName => write!(f, "__typename"),
            ExpectedType::Scalar(data_type) => write!(f, "{data_type}"),
            ExpectedType::Object(_) => write!(f, "Object"),
        }
    }
}
