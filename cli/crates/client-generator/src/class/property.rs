use core::fmt;
use std::borrow::Cow;

use crate::r#type::{Property, PropertyValue};

use super::Privacy;

#[derive(Debug)]
pub struct ClassProperty {
    inner: Property,
    privacy: Option<Privacy>,
}

impl ClassProperty {
    pub fn new(key: impl Into<Cow<'static, str>>, value: impl Into<PropertyValue>) -> Self {
        Self {
            inner: Property::new(key, value),
            privacy: None,
        }
    }

    #[must_use]
    pub fn optional(mut self) -> Self {
        self.inner = self.inner.optional();
        self
    }

    #[must_use]
    pub fn public(mut self) -> Self {
        self.privacy = Some(Privacy::Public);
        self
    }

    #[must_use]
    pub fn protected(mut self) -> Self {
        self.privacy = Some(Privacy::Protected);
        self
    }

    #[must_use]
    pub fn private(mut self) -> Self {
        self.privacy = Some(Privacy::Private);
        self
    }
}

impl fmt::Display for ClassProperty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(privacy) = self.privacy {
            write!(f, "{privacy} ")?;
        }

        self.inner.fmt(f)
    }
}
