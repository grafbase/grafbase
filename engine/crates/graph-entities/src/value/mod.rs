use grafbase_engine_value::{ConstValue, Name, Number};
use internment::ArcIntern;

mod value_serde;

/// A resolved GraphQL value, for example `1` or `"Hello World!"`.
///
/// This representation is designed to have a compact memory representation.
/// Every node in the response graph is allocated space for at least one of
/// these objects so it needs to be as small as possible.
///
/// This is in contrast to `ConstValue` which takes up 96 bytes of stack space
/// per entity.
///
/// We mostly acheive this by:
/// - Using `Vec` instead of `IndexMap`.  We have to give up constant time
///   lookup, but GraphQL objects don't tend to be massive so I'm hoping
///   that's fine.
/// - Using a `Vec<u8>` instead of `Bytes`.  This is less efficient for
///   cloning.  But hopefully we can just not do that.
///
/// It's possible we could entirely replace ConstValue with this at some
/// point, but I don't want to go to those lenghts right now.  Might take
/// a while and I can't confidently say this works for every case where
/// we use ConstValue.
///
/// [Reference](https://spec.graphql.org/June2018/#Value).
#[derive(Clone, Debug, Default)]
pub enum CompactValue {
    /// `null`.
    #[default]
    Null,
    /// A number.
    Number(Number),
    /// A string.
    String(String),
    /// A boolean.
    Boolean(bool),
    // /// A binary.
    Binary(Vec<u8>),
    /// An enum. These are typically in `SCREAMING_SNAKE_CASE`.
    Enum(ArcIntern<String>),
    /// A list of values.
    List(Vec<CompactValue>),
    /// An object. This is a map of keys to values.
    Object(Vec<(Name, CompactValue)>),
}

impl CompactValue {
    pub fn is_array(&self) -> bool {
        matches!(self, CompactValue::List(_))
    }
}

impl From<ConstValue> for CompactValue {
    fn from(value: ConstValue) -> Self {
        match value {
            ConstValue::Null => CompactValue::Null,
            ConstValue::Number(num) => CompactValue::Number(num),
            ConstValue::String(string) => CompactValue::String(string),
            ConstValue::Boolean(boolean) => CompactValue::Boolean(boolean),
            ConstValue::Binary(binary) => CompactValue::Binary(binary.to_vec()),
            ConstValue::Enum(en) => CompactValue::Enum(ArcIntern::new(en.to_string())),
            ConstValue::List(list) => CompactValue::List(list.into_iter().map(Into::into).collect()),
            ConstValue::Object(obj) => CompactValue::Object(obj.into_iter().map(|(k, v)| (k, v.into())).collect()),
        }
    }
}

// Note: would be nice to get rid of this conversion, as _usually_ we'd only need
// to go from ConstValue -> CompactValue.  But there's some query_planning stuff that works on
// ConstValue and I cba updating it right now so _for now_ we can keep this.
impl From<CompactValue> for ConstValue {
    fn from(value: CompactValue) -> Self {
        match value {
            CompactValue::Null => ConstValue::Null,
            CompactValue::Number(num) => ConstValue::Number(num),
            CompactValue::String(string) => ConstValue::String(string),
            CompactValue::Boolean(boolean) => ConstValue::Boolean(boolean),
            CompactValue::Binary(binary) => ConstValue::Binary(binary.into()),
            CompactValue::Enum(en) => ConstValue::Enum(Name::new(en.to_string())),
            CompactValue::List(list) => ConstValue::List(list.into_iter().map(Into::into).collect()),
            CompactValue::Object(obj) => ConstValue::Object(obj.into_iter().map(|(k, v)| (k, v.into())).collect()),
        }
    }
}

#[cfg(test)]
mod test {
    use grafbase_engine_value::ConstValue;

    use super::*;

    #[test]
    fn check_compact_value_is_compact() {
        assert!(std::mem::size_of::<CompactValue>() < std::mem::size_of::<ConstValue>());
        assert!(std::mem::size_of::<CompactValue>() <= 32);
    }
}
