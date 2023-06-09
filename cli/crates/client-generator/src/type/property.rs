use std::{borrow::Cow, fmt};

use super::{ObjectTypeDef, StaticType};

#[derive(Debug, Clone)]
pub struct Property {
    key: Cow<'static, str>,
    value: PropertyValue,
    optional: bool,
}

impl Property {
    pub fn new(key: impl Into<Cow<'static, str>>, value: impl Into<PropertyValue>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            optional: false,
        }
    }

    #[must_use]
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let optional = if self.optional { "?" } else { "" };
        write!(f, "{}{optional}: {}", self.key, self.value)
    }
}

#[derive(Debug, Clone)]
pub enum PropertyValue {
    Type(StaticType),
    Object(ObjectTypeDef),
}

impl From<StaticType> for PropertyValue {
    fn from(value: StaticType) -> Self {
        Self::Type(value)
    }
}

impl From<ObjectTypeDef> for PropertyValue {
    fn from(value: ObjectTypeDef) -> Self {
        Self::Object(value)
    }
}

impl fmt::Display for PropertyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyValue::Type(ident) => ident.fmt(f),
            PropertyValue::Object(obj) => obj.fmt(f),
        }
    }
}
