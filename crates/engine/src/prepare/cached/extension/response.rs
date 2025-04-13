use std::borrow::Cow;

use schema::{ExtensionInputValueRecord, InputValueSet};
use walker::Walk;

use crate::response::{ResponseObjectView, ResponseObjectsView};

use super::{ArgumentsContext, template::JsonContent};

pub struct ExtensionDirectiveArgumentsResponseObjectsView<'a> {
    pub(super) ctx: ArgumentsContext<'a>,
    pub(super) arguments: Vec<(&'a str, &'a ExtensionInputValueRecord)>,
    pub(super) response_objects_view: ResponseObjectsView<'a>,
}

impl ExtensionDirectiveArgumentsResponseObjectsView<'_> {
    pub(crate) fn iter(&self) -> impl Iterator<Item = ExtensionDirectiveArgumentsResponseObjectView<'_>> + '_ {
        self.response_objects_view.iter().map(move |response_object_view| {
            ExtensionDirectiveArgumentsResponseObjectView {
                ctx: &self.ctx,
                filtered_arguments: &self.arguments,
                response_object_view,
            }
        })
    }
}

pub struct ExtensionDirectiveArgumentsResponseObjectView<'a> {
    ctx: &'a ArgumentsContext<'a>,
    filtered_arguments: &'a [(&'a str, &'a ExtensionInputValueRecord)],
    response_object_view: ResponseObjectView<'a>,
}

impl serde::Serialize for ExtensionDirectiveArgumentsResponseObjectView<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self.filtered_arguments.iter().map(|(name, value)| {
            (
                name,
                ExtensionInputValueResponseObjectView {
                    ctx: self.ctx,
                    value,
                    response_object_view: self.response_object_view,
                },
            )
        }))
    }
}

struct ExtensionInputValueResponseObjectView<'a> {
    ctx: &'a ArgumentsContext<'a>,
    value: &'a ExtensionInputValueRecord,
    response_object_view: ResponseObjectView<'a>,
}

impl serde::Serialize for ExtensionInputValueResponseObjectView<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Self {
            ctx,
            value: v,
            response_object_view,
        } = *self;
        match v {
            ExtensionInputValueRecord::Null => serializer.serialize_none(),
            ExtensionInputValueRecord::String(id) => serializer.serialize_str(&ctx.schema[*id]),
            ExtensionInputValueRecord::EnumValue(id) => serializer.serialize_str(&ctx.schema[*id]),
            ExtensionInputValueRecord::Int(value) => serializer.serialize_i32(*value),
            ExtensionInputValueRecord::I64(value) => serializer.serialize_i64(*value),
            ExtensionInputValueRecord::U64(value) => serializer.serialize_u64(*value),
            ExtensionInputValueRecord::Float(value) => serializer.serialize_f64(*value),
            ExtensionInputValueRecord::Boolean(value) => serializer.serialize_bool(*value),
            ExtensionInputValueRecord::Map(map) => serializer.collect_map(map.iter().map(|(key, value)| {
                (
                    &ctx.schema[*key],
                    Self {
                        ctx,
                        value,
                        response_object_view,
                    },
                )
            })),
            ExtensionInputValueRecord::List(list) => serializer.collect_seq(list.iter().map(|value| Self {
                ctx,
                value,
                response_object_view,
            })),
            ExtensionInputValueRecord::InputValueSet(selection_set) => ctx
                .field_arguments
                .view(selection_set, ctx.variables)
                .serialize(serializer),
            ExtensionInputValueRecord::FieldSet(field_set) => {
                self.response_object_view.for_field_set(field_set).serialize(serializer)
            }
            ExtensionInputValueRecord::Template(id) => {
                let template = id.walk(ctx.schema);
                // FIXME: Should not serialize the whole arguments here. But for now that will
                // work.
                let args = serde_json::to_value(ctx.field_arguments.view(&InputValueSet::All, ctx.variables)).unwrap();
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
