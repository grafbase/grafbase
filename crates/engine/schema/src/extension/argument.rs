use std::cmp::Ordering;

use walker::Walk as _;

use crate::{ExtensionDirective, Schema, StringId};

use super::ExtensionInputValueRecord;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ExtensionDirectiveArgumentId(u32);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExtensionDirectiveArgumentRecord {
    pub name_id: StringId,
    pub value: ExtensionInputValueRecord,
    pub injection_stage: InjectionStage,
}

impl<'a> ExtensionDirective<'a> {
    pub fn argument_records(&self) -> &'a [ExtensionDirectiveArgumentRecord] {
        &self.schema[self.as_ref().argument_ids]
    }

    pub fn static_arguments(&self) -> StaticExtensionDirectiveArguments<'a> {
        StaticExtensionDirectiveArguments {
            schema: self.schema,
            ref_: self.argument_records(),
        }
    }
}

// When, at the earliest, can we compute the argument's value?
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InjectionStage {
    // No data injection, static data
    #[default]
    Static,
    // Injects data from the field arguments
    Query,
    // Injects data from the response such as fields
    Response,
}

impl PartialOrd for InjectionStage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InjectionStage {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Response, Self::Response) => Ordering::Equal,
            (Self::Response, Self::Query) | (Self::Response, Self::Static) => Ordering::Greater,
            (Self::Query, Self::Response) => Ordering::Less,
            (Self::Query, Self::Query) => Ordering::Equal,
            (Self::Query, Self::Static) => Ordering::Greater,
            (Self::Static, Self::Query) | (Self::Static, Self::Response) => Ordering::Less,
            (Self::Static, Self::Static) => Ordering::Equal,
        }
    }
}

#[derive(Clone, Copy)]
pub struct StaticExtensionDirectiveArguments<'a> {
    schema: &'a Schema,
    ref_: &'a [ExtensionDirectiveArgumentRecord],
}

impl serde::Serialize for StaticExtensionDirectiveArguments<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(
            self.ref_
                .iter()
                .filter(|arg| matches!(arg.injection_stage, InjectionStage::Static))
                .map(|arg| (&self.schema[arg.name_id], arg.value.walk(self.schema))),
        )
    }
}
