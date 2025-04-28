mod argument;
mod arguments;
mod data;
mod lookup;
mod typename;

pub(crate) use argument::*;
pub(crate) use arguments::*;
pub(crate) use data::*;
pub(crate) use lookup::*;
use operation::{Location, ResponseKey};
use schema::FieldDefinition;
pub(crate) use typename::*;

use super::{DataOrLookupField, DataOrLookupFieldId, ResponseObjectSetDefinitionId};

impl From<DataOrLookupFieldId> for u16 {
    fn from(value: DataOrLookupFieldId) -> Self {
        match value {
            DataOrLookupFieldId::Data(id) => u16::from(id) << 1 | 0x1,
            DataOrLookupFieldId::Lookup(id) => u16::from(id) << 1,
        }
    }
}

impl From<u16> for DataOrLookupFieldId {
    fn from(value: u16) -> Self {
        if value & 0x1 == 0x1 {
            DataOrLookupFieldId::Data((value >> 1).into())
        } else {
            DataOrLookupFieldId::Lookup((value >> 1).into())
        }
    }
}

impl<'a> DataOrLookupField<'a> {
    pub(crate) fn location(&self) -> Location {
        match self {
            DataOrLookupField::Data(field) => field.location,
            DataOrLookupField::Lookup(field) => field.location,
        }
    }
    pub(crate) fn subgraph_key(&self) -> ResponseKey {
        match self {
            DataOrLookupField::Data(field) => field.subgraph_key.unwrap_or(field.response_key),
            DataOrLookupField::Lookup(field) => field.subgraph_key,
        }
    }
    pub(crate) fn output_id(&self) -> Option<ResponseObjectSetDefinitionId> {
        match self {
            DataOrLookupField::Data(field) => field.output_id,
            DataOrLookupField::Lookup(field) => field.output_id,
        }
    }
    pub(crate) fn definition(&self) -> FieldDefinition<'a> {
        match self {
            DataOrLookupField::Data(field) => field.definition(),
            DataOrLookupField::Lookup(field) => field.definition(),
        }
    }
    pub(crate) fn arguments(&self) -> PlanFieldArguments<'a> {
        match self {
            DataOrLookupField::Data(field) => field.arguments(),
            DataOrLookupField::Lookup(field) => field.arguments(),
        }
    }
}
