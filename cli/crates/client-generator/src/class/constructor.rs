use std::{borrow::Cow, fmt};

use crate::{
    r#type::{Property, PropertyValue},
    Block, FunctionBody,
};

#[derive(Debug)]
pub struct Constructor {
    inner: FunctionBody,
}

impl Constructor {
    #[must_use]
    pub fn new(body: Block) -> Self {
        let inner = FunctionBody {
            name: Cow::Borrowed("constructor"),
            params: Vec::new(),
            returns: None,
            body,
        };

        Self { inner }
    }

    pub fn push_param(&mut self, key: impl Into<Cow<'static, str>>, value: impl Into<PropertyValue>) {
        self.inner.params.push(Property::new(key, value));
    }
}

impl fmt::Display for Constructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
