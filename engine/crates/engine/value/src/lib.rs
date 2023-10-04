//! Value for GraphQL. Used in the [`engine`] crate.

#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![allow(clippy::use_self)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::module_name_repetitions)]

mod deserializer;
mod macros;
mod serializer;
mod value_serde;
mod variables;

use std::{
    borrow::{Borrow, Cow},
    fmt::{self, Display, Formatter, Write},
    hash::Hash,
    ops::Deref,
    sync::Arc,
};

use bytes::Bytes;
pub use deserializer::{from_value, DeserializerError};
#[doc(hidden)]
pub use indexmap;
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub use serde_json::Number;
pub use serializer::{to_value, SerializerError};
pub use variables::Variables;

/// A GraphQL name.
///
/// [Reference](https://spec.graphql.org/June2018/#Name).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(Arc<str>);

impl Serialize for Name {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl Name {
    /// Create a new name.
    pub fn new(name: impl AsRef<str>) -> Self {
        Self(name.as_ref().into())
    }

    /// Get the name as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for Name {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl PartialEq<String> for Name {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}
impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}
impl PartialEq<Name> for String {
    fn eq(&self, other: &Name) -> bool {
        self == other.as_str()
    }
}
impl PartialEq<Name> for str {
    fn eq(&self, other: &Name) -> bool {
        other == self
    }
}
impl<'a> PartialEq<&'a str> for Name {
    fn eq(&self, other: &&'a str) -> bool {
        self == *other
    }
}
impl<'a> PartialEq<Name> for &'a str {
    fn eq(&self, other: &Name) -> bool {
        other == self
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(String::deserialize(deserializer)?.into_boxed_str().into()))
    }
}

/// A resolved GraphQL value, for example `1` or `"Hello World!"`.
///
/// It can be serialized and deserialized. Enums will be converted to strings. Attempting to
/// serialize `Upload` will fail, and `Enum` and `Upload` cannot be deserialized.
///
/// [Reference](https://spec.graphql.org/June2018/#Value).
#[derive(Clone, Debug, Eq)]
pub enum ConstValue {
    /// `null`.
    Null,
    /// A number.
    Number(Number),
    /// A string.
    String(String),
    /// A boolean.
    Boolean(bool),
    /// A binary.
    Binary(Bytes),
    /// An enum. These are typically in `SCREAMING_SNAKE_CASE`.
    Enum(Name),
    /// A list of values.
    List(Vec<ConstValue>),
    /// An object. This is a map of keys to values.
    Object(IndexMap<Name, ConstValue>),
}

impl Hash for ConstValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

        match self {
            ConstValue::Object(v) => {
                for (k, v) in v {
                    k.hash(state);
                    v.hash(state);
                }
            }
            ConstValue::Null => {}
            ConstValue::Number(v) => v.hash(state),
            ConstValue::String(v) => v.hash(state),
            ConstValue::Boolean(v) => v.hash(state),
            ConstValue::Binary(v) => v.hash(state),
            ConstValue::Enum(v) => v.hash(state),
            ConstValue::List(v) => v.hash(state),
        }
    }
}

impl ConstValue {
    /// Check if this is neither a null or a list of null
    pub fn is_null(&self) -> bool {
        match self {
            ConstValue::Null => true,
            ConstValue::List(vals) => !vals.iter().any(|x| !x.is_null()),
            _ => false,
        }
    }

    /// Check the [`ConstValue`] is an array
    pub fn is_array(&self) -> bool {
        matches!(self, ConstValue::List(_))
    }

    /// Check the [`ConstValue`] is an object
    pub fn is_object(&self) -> bool {
        matches!(self, ConstValue::Object(_))
    }

    /// If the `ConstValue` is a String, returns the associated str. Returns None
    /// otherwise.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConstValue::String(s) => Some(s),
            _ => None,
        }
    }
}

impl ConstValue {
    /// Returns a str of the kind of value this is.  Useful for error messages.
    pub fn kind_str(&self) -> &'static str {
        match self {
            ConstValue::Null => "null",
            ConstValue::Number(_) => "number",
            ConstValue::String(_) => "string",
            ConstValue::Boolean(_) => "boolean",
            ConstValue::Binary(_) => "binary",
            ConstValue::Enum(_) => "enum",
            ConstValue::List(_) => "list",
            ConstValue::Object(_) => "object",
        }
    }
}

impl PartialEq for ConstValue {
    fn eq(&self, other: &ConstValue) -> bool {
        match (self, other) {
            (ConstValue::Null, ConstValue::Null) => true,
            (ConstValue::Number(a), ConstValue::Number(b)) => a == b,
            (ConstValue::Boolean(a), ConstValue::Boolean(b)) => a == b,
            (ConstValue::String(a), ConstValue::String(b)) => a == b,
            (ConstValue::Enum(a), ConstValue::String(b)) => a == b,
            (ConstValue::String(a), ConstValue::Enum(b)) => a == b,
            (ConstValue::Enum(a), ConstValue::Enum(b)) => a == b,
            (ConstValue::Binary(a), ConstValue::Binary(b)) => a == b,
            (ConstValue::List(a), ConstValue::List(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                a.iter().zip(b.iter()).all(|(a, b)| a == b)
            }
            (ConstValue::Object(a), ConstValue::Object(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (a_key, a_value) in a.iter() {
                    if let Some(b_value) = b.get(a_key.as_str()) {
                        if b_value != a_value {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                true
            }
            _ => false,
        }
    }
}

impl From<()> for ConstValue {
    fn from((): ()) -> Self {
        ConstValue::Null
    }
}

macro_rules! from_integer {
    ($($ty:ident),*) => {
        $(
            impl From<$ty> for ConstValue {
                fn from(n: $ty) -> Self {
                    ConstValue::Number(n.into())
                }
            }
        )*
    };
}

from_integer!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl From<f32> for ConstValue {
    fn from(f: f32) -> Self {
        From::from(f64::from(f))
    }
}

impl From<f64> for ConstValue {
    fn from(f: f64) -> Self {
        Number::from_f64(f).map_or(ConstValue::Null, ConstValue::Number)
    }
}

impl From<bool> for ConstValue {
    fn from(value: bool) -> Self {
        ConstValue::Boolean(value)
    }
}

impl From<String> for ConstValue {
    fn from(value: String) -> Self {
        ConstValue::String(value)
    }
}

impl<'a> From<&'a str> for ConstValue {
    fn from(value: &'a str) -> Self {
        ConstValue::String(value.into())
    }
}

impl<'a> From<Cow<'a, str>> for ConstValue {
    fn from(f: Cow<'a, str>) -> Self {
        ConstValue::String(f.into_owned())
    }
}

impl<T: Into<ConstValue>> FromIterator<T> for ConstValue {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        ConstValue::List(iter.into_iter().map(Into::into).collect())
    }
}

impl<'a, T: Clone + Into<ConstValue>> From<&'a [T]> for ConstValue {
    fn from(f: &'a [T]) -> Self {
        ConstValue::List(f.iter().cloned().map(Into::into).collect())
    }
}

impl<T: Into<ConstValue>> From<Vec<T>> for ConstValue {
    fn from(f: Vec<T>) -> Self {
        ConstValue::List(f.into_iter().map(Into::into).collect())
    }
}

impl From<IndexMap<Name, ConstValue>> for ConstValue {
    fn from(f: IndexMap<Name, ConstValue>) -> Self {
        ConstValue::Object(f)
    }
}

impl ConstValue {
    /// Convert this `ConstValue` into a `Value`.
    #[must_use]
    pub fn into_value(self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Number(num) => {
                // We force it to be a f64 in the internal representation to generate the
                // appropriate ArrowSchema
                Value::Number(Number::from_f64(num.as_f64().expect("can't fail")).expect("can't fail"))
            }
            Self::String(s) => Value::String(s),
            Self::Boolean(b) => Value::Boolean(b),
            Self::Binary(bytes) => Value::Binary(bytes),
            Self::Enum(v) => Value::Enum(v),
            Self::List(items) => Value::List(items.into_iter().map(ConstValue::into_value).collect()),
            Self::Object(map) => Value::Object(map.into_iter().map(|(key, value)| (key, value.into_value())).collect()),
        }
    }

    /// Attempt to convert the value into JSON. This is equivalent to the `TryFrom` implementation.
    ///
    /// # Errors
    ///
    /// Fails if serialization fails (see enum docs for more info).
    pub fn into_json(self) -> serde_json::Result<serde_json::Value> {
        self.try_into()
    }

    /// Attempt to convert JSON into a value. This is equivalent to the `TryFrom` implementation.
    ///
    /// # Errors
    ///
    /// Fails if deserialization fails (see enum docs for more info).
    pub fn from_json(json: serde_json::Value) -> serde_json::Result<Self> {
        json.try_into()
    }
}

impl Default for ConstValue {
    fn default() -> Self {
        Self::Null
    }
}

impl Display for ConstValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(num) => write!(f, "{}", *num),
            Self::String(val) => write_quoted(val, f),
            Self::Boolean(true) => f.write_str("true"),
            Self::Boolean(false) => f.write_str("false"),
            Self::Binary(bytes) => write_binary(bytes, f),
            Self::Null => f.write_str("null"),
            Self::Enum(name) => f.write_str(name),
            Self::List(items) => write_list(items, f),
            Self::Object(map) => write_object(map, f),
        }
    }
}

impl TryFrom<serde_json::Value> for ConstValue {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        Self::deserialize(value)
    }
}

impl TryFrom<ConstValue> for serde_json::Value {
    type Error = serde_json::Error;
    fn try_from(value: ConstValue) -> Result<Self, Self::Error> {
        serde_json::to_value(value)
    }
}

/// A GraphQL value, for example `1`, `$name` or `"Hello World!"`. This is
/// [`ConstValue`](enum.ConstValue.html) with variables.
///
/// It can be serialized and deserialized. Enums will be converted to strings. Attempting to
/// serialize `Upload` or `Variable` will fail, and `Enum`, `Upload` and `Variable` cannot be
/// deserialized.
///
/// [Reference](https://spec.graphql.org/June2018/#Value).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    /// A variable, without the `$`.
    Variable(Name),
    /// `null`.
    Null,
    /// A number.
    Number(Number),
    /// A string.
    String(String),
    /// A boolean.
    Boolean(bool),
    /// A binary.
    Binary(Bytes),
    /// An enum. These are typically in `SCREAMING_SNAKE_CASE`.
    Enum(Name),
    /// A list of values.
    List(Vec<Value>),
    /// An object. This is a map of keys to values.
    Object(IndexMap<Name, Value>),
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Object(a) => a.as_slice().hash(state),
            Self::Null => false.hash(state),
            Self::Number(n) => n.hash(state),
            Self::String(n) => n.hash(state),
            Self::Boolean(n) => n.hash(state),
            Self::Binary(n) => n.hash(state),
            Self::List(n) => n.hash(state),
            Self::Variable(n) | Self::Enum(n) => n.hash(state),
        }
    }
}

impl Value {
    /// Attempt to convert the value into a const value by using a function to get a variable.
    pub fn into_const_with<E>(self, mut f: impl FnMut(Name) -> Result<ConstValue, E>) -> Result<ConstValue, E> {
        self.into_const_with_mut(&mut f)
    }

    fn into_const_with_mut<E>(self, f: &mut impl FnMut(Name) -> Result<ConstValue, E>) -> Result<ConstValue, E> {
        Ok(match self {
            Self::Variable(name) => f(name)?,
            Self::Null => ConstValue::Null,
            Self::Number(num) => ConstValue::Number(num),
            Self::String(s) => ConstValue::String(s),
            Self::Boolean(b) => ConstValue::Boolean(b),
            Self::Binary(v) => ConstValue::Binary(v),
            Self::Enum(v) => ConstValue::Enum(v),
            Self::List(items) => ConstValue::List(
                items
                    .into_iter()
                    .map(|value| value.into_const_with_mut(f))
                    .collect::<Result<_, _>>()?,
            ),
            Self::Object(map) => ConstValue::Object(
                map.into_iter()
                    .map(|(key, value)| Ok((key, value.into_const_with_mut(f)?)))
                    .collect::<Result<_, _>>()?,
            ),
        })
    }

    /// Attempt to convert the value into a const value.
    ///
    /// Will fail if the value contains variables.
    #[must_use]
    pub fn into_const(self) -> Option<ConstValue> {
        self.into_const_with(|_| Err(())).ok()
    }

    /// Attempt to convert the value into JSON. This is equivalent to the `TryFrom` implementation.
    ///
    /// # Errors
    ///
    /// Fails if serialization fails (see enum docs for more info).
    pub fn into_json(self) -> serde_json::Result<serde_json::Value> {
        self.try_into()
    }

    /// Attempt to convert JSON into a value. This is equivalent to the `TryFrom` implementation.
    ///
    /// # Errors
    ///
    /// Fails if deserialization fails (see enum docs for more info).
    pub fn from_json(json: serde_json::Value) -> serde_json::Result<Self> {
        json.try_into()
    }

    /// Attempt to convert the value into a u64 integer.
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Number(num) => num.as_u64(),
            _ => None,
        }
    }

    /// Attempt to convert the value into a string slice.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            Value::Enum(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Attempt to convert the value into a string.
    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.to_string()),
            Value::Enum(s) => Some(s.to_string()),
            _ => None,
        }
    }

    /// Attempt to convert the value into a slice.
    pub fn as_slice(&self) -> Option<&[Value]> {
        match self {
            Value::List(lst) => Some(lst),
            _ => None,
        }
    }

    /// Attempt to convert the value into an object.
    pub fn as_object(&self) -> Option<&IndexMap<Name, Value>> {
        match self {
            Value::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Returns an iterator over the variables that are used in this Value
    pub fn variables_used(&self) -> VariableIterator<'_> {
        VariableIterator::new(self)
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variable(name) => write!(f, "${name}"),
            Self::Number(num) => write!(f, "{}", *num),
            Self::String(val) => write_quoted(val, f),
            Self::Boolean(true) => f.write_str("true"),
            Self::Boolean(false) => f.write_str("false"),
            Self::Binary(bytes) => write_binary(bytes, f),
            Self::Null => f.write_str("null"),
            Self::Enum(name) => f.write_str(name),
            Self::List(items) => write_list(items, f),
            Self::Object(map) => write_object(map, f),
        }
    }
}

impl From<ConstValue> for Value {
    fn from(value: ConstValue) -> Self {
        value.into_value()
    }
}

impl TryFrom<serde_json::Value> for Value {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        Self::deserialize(value)
    }
}
impl TryFrom<Value> for serde_json::Value {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::to_value(value)
    }
}

fn write_quoted(s: &str, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_char('"')?;
    for c in s.chars() {
        match c {
            '\r' => f.write_str("\\r"),
            '\n' => f.write_str("\\n"),
            '\t' => f.write_str("\\t"),
            '"' => f.write_str("\\\""),
            '\\' => f.write_str("\\\\"),
            c if c.is_control() => write!(f, "\\u{:04}", c as u32),
            c => f.write_char(c),
        }?;
    }
    f.write_char('"')
}

fn write_binary(bytes: &[u8], f: &mut Formatter<'_>) -> fmt::Result {
    f.write_char('[')?;
    let mut iter = bytes.iter().copied();
    if let Some(value) = iter.next() {
        value.fmt(f)?;
    }
    for value in iter {
        f.write_char(',')?;
        value.fmt(f)?;
    }
    f.write_char(']')
}

fn write_list<T: Display>(list: impl IntoIterator<Item = T>, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_char('[')?;
    let mut iter = list.into_iter();
    if let Some(item) = iter.next() {
        item.fmt(f)?;
    }
    for item in iter {
        f.write_char(',')?;
        item.fmt(f)?;
    }
    f.write_char(']')
}

fn write_object<K: Display, V: Display>(
    object: impl IntoIterator<Item = (K, V)>,
    f: &mut Formatter<'_>,
) -> fmt::Result {
    f.write_char('{')?;
    let mut iter = object.into_iter();
    if let Some((name, value)) = iter.next() {
        write!(f, "{name}: {value}")?;
    }
    for (name, value) in iter {
        f.write_char(',')?;
        write!(f, "{name}: {value}")?;
    }
    f.write_char('}')
}

/// Iterator over the variables that are used inside a Value
pub struct VariableIterator<'a> {
    values: Vec<&'a Value>,
}

impl<'a> VariableIterator<'a> {
    fn new(value: &'a Value) -> Self {
        VariableIterator { values: vec![value] }
    }
}

impl<'a> Iterator for VariableIterator<'a> {
    type Item = &'a Name;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.values.pop()? {
                Value::Variable(name) => return Some(name),
                Value::Null
                | Value::Number(_)
                | Value::String(_)
                | Value::Boolean(_)
                | Value::Binary(_)
                | Value::Enum(_) => {}
                Value::List(values) => self.values.extend(values.iter()),
                Value::Object(obj) => self.values.extend(obj.values()),
            }
        }
    }
}
