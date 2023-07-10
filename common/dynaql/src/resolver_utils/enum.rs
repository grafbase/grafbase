use graph_entities::{CompactValue, ResponseNodeId};
use internment::ArcIntern;

use crate::{
    context::ContextSelectionSet, InputValueError, InputValueResult, LegacyInputType, Name, Value,
};

/// A variant of an enum.
pub struct EnumItem<T> {
    /// The name of the variant.
    pub name: &'static str,
    /// The value of the variant.
    pub value: T,
}

/// A GraphQL enum.
pub trait LegacyEnumType: Sized + Eq + Send + Copy + 'static {
    /// Get a list of possible variants of the enum and their values.
    fn items() -> &'static [EnumItem<Self>];
}

/// Parse a value as an enum value.
///
/// This can be used to implement `InputType::parse`.
pub fn parse_enum<T: LegacyEnumType + LegacyInputType>(value: Value) -> InputValueResult<T> {
    let value = match &value {
        Value::Enum(s) => s,
        Value::String(s) => s.as_str(),
        _ => return Err(InputValueError::expected_type(value)),
    };

    T::items()
        .iter()
        .find(|item| item.name == value)
        .map(|item| item.value)
        .ok_or_else(|| {
            InputValueError::custom(format_args!(
                r#"Enumeration type does not contain value "{value}"."#,
            ))
        })
}

/// Convert the enum value into a GraphQL value.
///
/// This can be used to implement `InputType::to_value` or `OutputType::resolve`.
pub fn enum_value<T: LegacyEnumType>(value: T) -> Value {
    let item = T::items().iter().find(|item| item.value == value).unwrap();
    Value::Enum(Name::new(item.name))
}

pub async fn enum_value_node<'a, T: LegacyEnumType>(
    ctx: &ContextSelectionSet<'a>,
    value: T,
) -> ResponseNodeId {
    let item = T::items().iter().find(|item| item.value == value).unwrap();

    let mut response_graph = ctx.response_graph.write().await;
    response_graph.insert_node(CompactValue::Enum(ArcIntern::new(item.name.to_string())))
}
