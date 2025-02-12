use operation::Variables;
use schema::{ExtensionDirectiveArgumentRecord, ExtensionInputValueRecord, InjectionStage, Schema};

use super::PartitionFieldArguments;

#[derive(Clone, Copy)]
pub struct ExtensionDirectiveArgumentsQueryView<'a> {
    pub(in crate::prepare::cached::query_plan) schema: &'a Schema,
    pub(in crate::prepare::cached::query_plan) argument_records: &'a [ExtensionDirectiveArgumentRecord],
    pub(in crate::prepare::cached::query_plan) field_arguments: PartitionFieldArguments<'a>,
    pub(in crate::prepare::cached::query_plan) variables: &'a Variables,
}

impl serde::Serialize for ExtensionDirectiveArgumentsQueryView<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(
            self.argument_records
                .iter()
                .filter(|arg| arg.injection_stage <= InjectionStage::Query)
                .map(|arg| {
                    (
                        &self.schema[arg.name_id],
                        ExtensionInputValueQueryView {
                            ctx: self,
                            value: &arg.value,
                        },
                    )
                }),
        )
    }
}

#[derive(Clone, Copy)]
struct ExtensionInputValueQueryView<'a> {
    pub ctx: &'a ExtensionDirectiveArgumentsQueryView<'a>,
    pub value: &'a ExtensionInputValueRecord,
}

impl serde::Serialize for ExtensionInputValueQueryView<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Self { ctx, value } = *self;
        match value {
            ExtensionInputValueRecord::Null => serializer.serialize_none(),
            ExtensionInputValueRecord::String(id) => serializer.serialize_str(&ctx.schema[*id]),
            ExtensionInputValueRecord::EnumValue(id) => serializer.serialize_str(&ctx.schema[*id]),
            ExtensionInputValueRecord::Int(value) => serializer.serialize_i32(*value),
            ExtensionInputValueRecord::BigInt(value) => serializer.serialize_i64(*value),
            ExtensionInputValueRecord::U64(value) => serializer.serialize_u64(*value),
            ExtensionInputValueRecord::Float(value) => serializer.serialize_f64(*value),
            ExtensionInputValueRecord::Boolean(value) => serializer.serialize_bool(*value),
            ExtensionInputValueRecord::Map(map) => {
                serializer.collect_map(map.iter().map(|(key, value)| (&ctx.schema[*key], Self { ctx, value })))
            }
            ExtensionInputValueRecord::List(list) => {
                serializer.collect_seq(list.iter().map(|value| Self { ctx, value }))
            }
            ExtensionInputValueRecord::InputValueSet(selection_set) => ctx
                .field_arguments
                .view(selection_set, ctx.variables)
                .serialize(serializer),
            ExtensionInputValueRecord::FieldSet(_) => {
                unreachable!("Invariant broken, cannot be a static value.")
            }
        }
    }
}
