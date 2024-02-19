use crate::{InputValue, RawInputValue, RawInputValueId, RawInputValues, SchemaInputValueId, SchemaWalker, StringId};

/// Walker over a RawInputValue providing Serialize, Deserialize, Display and Into<InputValue<'_>>.
#[derive(Clone, Copy)]
pub struct RawInputValueWalker<'ctx, Ctx: RawInputValuesContext<'ctx>> {
    pub(super) ctx: Ctx,
    pub(super) value: &'ctx RawInputValue<Ctx::Str>,
}

impl<'ctx, Ctx> RawInputValueWalker<'ctx, Ctx>
where
    Ctx: RawInputValuesContext<'ctx>,
{
    pub(super) fn walk(&self, value: &'ctx RawInputValue<Ctx::Str>) -> Self {
        Self { ctx: self.ctx, value }
    }

    /// Undefined values within InputObjects are silently removed according to the GraphQL spec.
    /// But we can't remove an undefined argument
    pub fn is_undefined(&self) -> bool {
        match self.value {
            RawInputValue::Undefined => true,
            RawInputValue::Ref(id) => self.ctx.walk(*id).is_undefined(),
            // I don't think this should ever happen, so should maybe be an unreachable!()
            RawInputValue::SchemaRef(id) => self.ctx.schema_walk(*id).is_undefined(),
            _ => false,
        }
    }
}

/// Context providing all the necessary information for the RawInputValueWalker. There are two
/// cases:
/// - schema input values, for which the context is simply a SchemaWalker. Strings are all interned
///   and identified by a Stringid.
/// - operation input values. Strings are all just Box<str>.
pub trait RawInputValuesContext<'ctx>: Clone + Copy + 'ctx {
    type Str: 'static;

    fn schema_walker(&self) -> &SchemaWalker<'ctx, ()>;
    fn get_str(&self, s: &'ctx Self::Str) -> &'ctx str;
    fn input_values(&self) -> &'ctx RawInputValues<Self::Str>;
    /// Defines how to display a InputValue::Ref(id). Used to show variable names
    /// for operation input values when variables aren't bound yet.
    fn input_value_ref_display(&self, id: RawInputValueId<Self::Str>) -> impl std::fmt::Display + 'ctx;

    fn walk(&self, id: RawInputValueId<Self::Str>) -> RawInputValueWalker<'ctx, Self> {
        RawInputValueWalker {
            ctx: *self,
            value: &self.input_values()[id],
        }
    }

    fn schema_walk(&self, id: SchemaInputValueId) -> RawInputValueWalker<'ctx, SchemaWalker<'ctx, ()>> {
        let ctx = *self.schema_walker();
        let value = &ctx.input_values()[id];
        RawInputValueWalker { ctx, value }
    }
}

impl<'ctx> RawInputValuesContext<'ctx> for SchemaWalker<'ctx, ()> {
    type Str = StringId;

    fn schema_walker(&self) -> &SchemaWalker<'ctx, ()> {
        self
    }

    fn get_str(&self, s: &StringId) -> &'ctx str {
        &self.schema[*s]
    }

    fn input_values(&self) -> &'ctx RawInputValues<StringId> {
        &self.schema.input_values
    }

    fn input_value_ref_display(&self, id: RawInputValueId<StringId>) -> impl std::fmt::Display + 'ctx {
        RawInputValuesContext::walk(self, id)
    }
}

/// A RawInputValue isn't very friendly to manipulate directly and it becomes even more tricky when
/// one needs to handle default values coming from the schema and request input values.
/// So when one needs to do more complex processing than Serialize, it's best to just manipulate an
/// InputValue.
impl<'ctx, Ctx> From<RawInputValueWalker<'ctx, Ctx>> for InputValue<'ctx>
where
    Ctx: RawInputValuesContext<'ctx>,
{
    fn from(walker: RawInputValueWalker<'ctx, Ctx>) -> Self {
        match walker.value {
            RawInputValue::Null | RawInputValue::Undefined => InputValue::Null,
            RawInputValue::String(s) | RawInputValue::UnknownEnumValue(s) => InputValue::String(walker.ctx.get_str(s)),
            RawInputValue::EnumValue(id) => InputValue::EnumValue(*id),
            RawInputValue::Int(n) => InputValue::Int(*n),
            RawInputValue::BigInt(n) => InputValue::BigInt(*n),
            RawInputValue::Float(f) => InputValue::Float(*f),
            RawInputValue::Boolean(b) => InputValue::Boolean(*b),
            RawInputValue::InputObject(ids) => {
                let mut fields = Vec::with_capacity(ids.len());
                for (input_value_definition_id, value) in &walker.ctx.input_values()[*ids] {
                    fields.push((*input_value_definition_id, walker.walk(value).into()));
                }
                InputValue::InputObject(fields.into_boxed_slice())
            }
            RawInputValue::List(ids) => {
                let mut values = Vec::with_capacity(ids.len());
                for id in *ids {
                    values.push(walker.ctx.walk(id).into());
                }
                InputValue::List(values.into_boxed_slice())
            }
            RawInputValue::Map(ids) => {
                let mut key_values = Vec::with_capacity(ids.len());
                for (key, value) in &walker.ctx.input_values()[*ids] {
                    key_values.push((walker.ctx.get_str(key), walker.walk(value).into()));
                }
                InputValue::Map(key_values.into_boxed_slice())
            }
            RawInputValue::U64(n) => InputValue::U64(*n),
            RawInputValue::Ref(id) => walker.ctx.walk(*id).into(),
            RawInputValue::SchemaRef(id) => walker.ctx.schema_walk(*id).into(),
        }
    }
}
