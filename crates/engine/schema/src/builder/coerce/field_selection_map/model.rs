use crate::{FieldDefinitionId, SchemaInputValueId};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoundSelectedValue<Id> {
    pub alternatives: Vec<BoundSelectedValueEntry<Id>>,
}

impl<Id> BoundSelectedValue<Id> {
    pub fn into_single(self) -> Option<BoundSelectedValueEntry<Id>> {
        if self.alternatives.len() == 1 {
            Some(self.alternatives.into_iter().next().unwrap())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BoundSelectedValueEntry<Id> {
    Identity,
    Path(BoundPath),
    Object {
        path: Option<BoundPath>,
        object: BoundSelectedObjectValue<Id>,
    },
    List {
        path: Option<BoundPath>,
        list: BoundSelectedListValue<Id>,
    },
}

impl<Id> BoundSelectedValueEntry<Id> {
    pub fn into_path(self) -> Option<BoundPath> {
        match self {
            BoundSelectedValueEntry::Path(path) => Some(path),
            _ => None,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BoundPath(pub Vec<FieldDefinitionId>);

impl std::ops::Deref for BoundPath {
    type Target = [FieldDefinitionId];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BoundPath {
    pub fn into_single(self) -> Option<FieldDefinitionId> {
        if self.0.len() == 1 {
            Some(self.0.into_iter().next().unwrap())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoundSelectedObjectValue<Id> {
    pub fields: Vec<BoundSelectedObjectField<Id>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoundSelectedObjectField<Id> {
    pub id: Id,
    pub value: SelectedValueOrField<Id>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SelectedValueOrField<Id> {
    Value(BoundSelectedValue<Id>),
    Field(FieldDefinitionId),
    DefaultValue(SchemaInputValueId),
}

impl<Id> SelectedValueOrField<Id> {
    pub fn into_value(self) -> Option<BoundSelectedValue<Id>> {
        if let SelectedValueOrField::Value(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoundSelectedListValue<Id>(pub BoundSelectedValue<Id>);
