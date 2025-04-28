use std::borrow::Cow;

use schema::{ExtensionDirective, ExtensionInputValueRecord, InjectionStage, InputValueSet};
use walker::Walk;

use super::{ArgumentsContext, template::JsonContent};

#[derive(Clone, Copy)]
pub struct ExtensionDirectiveArgumentsQueryView<'a> {
    pub(crate) ctx: ArgumentsContext<'a>,
    pub(crate) directive: ExtensionDirective<'a>,
}

impl serde::Serialize for ExtensionDirectiveArgumentsQueryView<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(
            self.directive
                .arguments_with_stage(|stage| stage <= InjectionStage::Query)
                .map(|(name, value)| (name, ExtensionInputValueQueryView { ctx: &self.ctx, value })),
        )
    }
}

#[derive(Clone, Copy)]
struct ExtensionInputValueQueryView<'a> {
    ctx: &'a ArgumentsContext<'a>,
    value: &'a ExtensionInputValueRecord,
}

impl serde::Serialize for ExtensionInputValueQueryView<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Self { ctx, value } = *self;
        match value {
            ExtensionInputValueRecord::Null => serializer.serialize_none(),
            ExtensionInputValueRecord::String(id) => serializer.serialize_str(&ctx.schema[*id]),
            ExtensionInputValueRecord::EnumValue(id) => serializer.serialize_str(&ctx.schema[*id]),
            ExtensionInputValueRecord::Int(value) => serializer.serialize_i32(*value),
            ExtensionInputValueRecord::I64(value) => serializer.serialize_i64(*value),
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
                .query_view(selection_set, ctx.variables)
                .serialize(serializer),
            ExtensionInputValueRecord::FieldSet(_) => {
                unreachable!("Invariant broken, cannot be available from the operation alone.")
            }
            ExtensionInputValueRecord::Template(id) => {
                let template = id.walk(ctx.schema);
                // FIXME: Should not serialize the whole arguments here. But for now that will
                // work.
                let args =
                    serde_json::to_value(ctx.field_arguments.query_view(&InputValueSet::All, ctx.variables)).unwrap();
                template
                    .inner
                    .render(&JsonContent {
                        value: Cow::Owned(serde_json::json!({"args": args})),
                        escaping: template.escaping,
                    })
                    .serialize(serializer)
            }
        }
    }
}
