use std::cmp::Ordering;

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

impl InjectionStage {
    pub fn is_static(self) -> bool {
        matches!(self, Self::Static)
    }

    pub fn is_query(self) -> bool {
        matches!(self, Self::Query)
    }

    pub fn is_response(self) -> bool {
        matches!(self, Self::Response)
    }
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
pub struct ExtensionDirectiveArgumentsStaticView<'a> {
    pub(super) directive: ExtensionDirective<'a>,
}

impl serde::Serialize for ExtensionDirectiveArgumentsStaticView<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(
            self.directive
                .arguments_with_stage(|stage| matches!(stage, InjectionStage::Static))
                .map(|(name, value)| {
                    (
                        name,
                        ExtensionInputValueStaticView {
                            schema: self.directive.schema,
                            ref_: value,
                        },
                    )
                }),
        )
    }
}

#[derive(Clone, Copy)]
struct ExtensionInputValueStaticView<'a> {
    schema: &'a Schema,
    ref_: &'a ExtensionInputValueRecord,
}

impl std::fmt::Debug for ExtensionInputValueStaticView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionInputValueStaticView").finish_non_exhaustive()
    }
}

impl serde::Serialize for ExtensionInputValueStaticView<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Self { schema, ref_ } = *self;
        match ref_ {
            ExtensionInputValueRecord::Null => serializer.serialize_none(),
            ExtensionInputValueRecord::String(id) => serializer.serialize_str(&schema[*id]),
            ExtensionInputValueRecord::EnumValue(id) => serializer.serialize_str(&schema[*id]),
            ExtensionInputValueRecord::Int(value) => serializer.serialize_i32(*value),
            ExtensionInputValueRecord::BigInt(value) => serializer.serialize_i64(*value),
            ExtensionInputValueRecord::U64(value) => serializer.serialize_u64(*value),
            ExtensionInputValueRecord::Float(value) => serializer.serialize_f64(*value),
            ExtensionInputValueRecord::Boolean(value) => serializer.serialize_bool(*value),
            ExtensionInputValueRecord::Map(map) => {
                serializer.collect_map(map.iter().map(|(key, ref_)| (&schema[*key], Self { schema, ref_ })))
            }
            ExtensionInputValueRecord::List(list) => {
                serializer.collect_seq(list.iter().map(|ref_| Self { schema, ref_ }))
            }
            ExtensionInputValueRecord::FieldSet(_)
            | ExtensionInputValueRecord::InputValueSet(_)
            | ExtensionInputValueRecord::Template(_) => {
                unreachable!("Invariant broken, cannot be a static value.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injection_stage_ordering() {
        assert!(InjectionStage::Static < InjectionStage::Query);
        assert!(InjectionStage::Query < InjectionStage::Response);
        assert!(InjectionStage::Static < InjectionStage::Response);
    }
}
