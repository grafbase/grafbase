use crate::FieldDefinitionId;

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

impl<Id> BoundSelectedValueEntry<Id> {
    pub fn into_object(self) -> Option<BoundSelectedObjectValue<Id>> {
        match self {
            BoundSelectedValueEntry::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn into_path(self) -> Option<BoundPath> {
        match self {
            BoundSelectedValueEntry::Path(path) => Some(path),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BoundPath(pub Vec<FieldDefinitionId>);

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
    pub field: Id,
    pub value: Option<BoundSelectedValue<Id>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoundSelectedListValue<Id>(pub BoundSelectedValue<Id>);
