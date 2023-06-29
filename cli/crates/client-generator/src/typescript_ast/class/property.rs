use core::fmt;
use std::borrow::Cow;

use crate::typescript_ast::{Property, PropertyValue};

use super::Privacy;

pub struct ClassProperty<'a> {
    inner: Property<'a>,
    privacy: Option<Privacy>,
}

#[allow(dead_code)]
impl<'a> ClassProperty<'a> {
    pub fn new(key: impl Into<Cow<'a, str>>, value: impl Into<PropertyValue<'a>>) -> Self {
        Self {
            inner: Property::new(key, value),
            privacy: None,
        }
    }

    pub fn optional(&mut self) {
        self.inner.optional();
    }

    pub fn public(&mut self) {
        Some(Privacy::Public);
    }

    pub fn protected(&mut self) {
        self.privacy = Some(Privacy::Protected);
    }

    pub fn private(&mut self) {
        self.privacy = Some(Privacy::Private);
    }
}

impl<'a> fmt::Display for ClassProperty<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(privacy) = self.privacy {
            write!(f, "{privacy} ")?;
        }

        self.inner.fmt(f)
    }
}
