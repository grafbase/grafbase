use std::{borrow::Cow, fmt};

use crate::{
    r#type::{Property, PropertyValue},
    Block, FunctionBody,
};

pub struct Constructor<'a> {
    inner: FunctionBody<'a>,
}

#[allow(dead_code)]
impl<'a> Constructor<'a> {
    #[must_use]
    pub fn new(body: Block<'a>) -> Self {
        let inner = FunctionBody {
            name: Cow::Borrowed("constructor"),
            params: Vec::new(),
            returns: None,
            body,
        };

        Self { inner }
    }

    pub fn push_param(&mut self, key: impl Into<Cow<'a, str>>, value: impl Into<PropertyValue<'a>>) {
        self.inner.params.push(Property::new(key, value));
    }
}

impl<'a> fmt::Display for Constructor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
