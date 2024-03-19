use schema::{InputValue, InputValueSerdeError, RawInputValuesContext, SchemaWalker};
use serde::{de::Visitor, forward_to_deserialize_any};

use crate::{
    operation::{OpInputValueId, OpInputValues},
    plan::OperationPlan,
};

use super::PlanWalker;

pub type PlanInputValue<'a> = PlanWalker<'a, OpInputValueId, ()>;

impl<'a> PlanInputValue<'a> {
    fn as_ctx(&self) -> PlanInputValueContext<'a> {
        PlanInputValueContext {
            schema_walker: self.schema_walker,
            operation_plan: self.operation_plan,
            input_values: self.input_values,
        }
    }

    pub fn is_undefined(&self) -> bool {
        self.as_ctx().walk(self.item).is_undefined()
    }

    pub fn id(&self) -> OpInputValueId {
        self.item
    }
}

impl<'a> From<PlanInputValue<'a>> for InputValue<'a> {
    fn from(value: PlanInputValue<'a>) -> Self {
        value.as_ctx().walk(value.item).into()
    }
}

impl<'a> serde::Serialize for PlanInputValue<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_ctx().walk(self.item).serialize(serializer)
    }
}

impl<'de> serde::Deserializer<'de> for PlanInputValue<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.as_ctx().walk(self.item).deserialize_any(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.as_ctx().walk(self.item).deserialize_option(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier
    }
}

impl std::fmt::Debug for PlanInputValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for PlanInputValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ctx().walk(self.item).fmt(f)
    }
}

#[derive(Clone, Copy)]
struct PlanInputValueContext<'a> {
    schema_walker: SchemaWalker<'a, ()>,
    operation_plan: &'a OperationPlan,
    input_values: Option<&'a OpInputValues>,
}

impl<'ctx> RawInputValuesContext<'ctx> for PlanInputValueContext<'ctx> {
    type Str = Box<str>;

    fn schema_walker(&self) -> &schema::SchemaWalker<'ctx, ()> {
        &self.schema_walker
    }

    fn get_str(&self, s: &'ctx Box<str>) -> &'ctx str {
        s.as_ref()
    }

    fn input_values(&self) -> &'ctx schema::RawInputValues<Box<str>> {
        self.input_values.unwrap_or(&self.operation_plan.operation.input_values)
    }

    fn input_value_ref_display(&self, id: schema::RawInputValueId<Box<str>>) -> impl std::fmt::Display + 'ctx {
        RefDisplay { ctx: *self, id }
    }
}

struct RefDisplay<'ctx> {
    ctx: PlanInputValueContext<'ctx>,
    id: OpInputValueId,
}

impl<'ctx> std::fmt::Display for RefDisplay<'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Variables weren't injected yet.
        if self.ctx.input_values.is_none() {
            if let Some(variable) = self
                .ctx
                .operation_plan
                .variable_definitions
                .iter()
                .find(|var| var.future_input_value_id == self.id)
            {
                return write!(f, "${}", variable.name);
            }
        }
        self.ctx.walk(self.id).fmt(f)
    }
}
