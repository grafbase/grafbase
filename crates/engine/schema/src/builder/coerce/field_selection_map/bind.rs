use crate::FieldDefinitionId;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoundSelectedValue<Id> {
    pub alternatives: Vec<BoundSelectedValueEntry<Id>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BoundSelectedValueEntry<Id> {
    Path(BoundPath),
    ObjectWithPath {
        path: BoundPath,
        object: BoundSelectedObjectValue<Id>,
    },
    ListWithPath {
        path: BoundPath,
        list: BoundSelectedListValue<Id>,
    },
    Object(BoundSelectedObjectValue<Id>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BoundPath(pub Vec<FieldDefinitionId>);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoundSelectedObjectValue<Id> {
    pub fields: Vec<BoundSelectedObjectField<Id>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoundSelectedObjectField<Id> {
    pub field: Id,
    pub value: Option<BoundSelectedValue<Id>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoundSelectedListValue<Id>(pub BoundSelectedValue<Id>);
