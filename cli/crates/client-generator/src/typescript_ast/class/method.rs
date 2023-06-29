use std::{borrow::Cow, fmt};

use crate::typescript_ast::{Block, FunctionBody, Property, PropertyValue, TypeKind};

use super::Privacy;

pub struct Method<'a> {
    inner: FunctionBody<'a>,
    privacy: Option<Privacy>,
}

#[allow(dead_code)]
impl<'a> Method<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>, body: Block<'a>) -> Self {
        let inner = FunctionBody {
            name: name.into(),
            params: Vec::new(),
            returns: None,
            body,
        };

        Self { inner, privacy: None }
    }

    pub fn returns(mut self, r#type: impl Into<TypeKind<'a>>) -> Self {
        self.inner.returns = Some(r#type.into());
        self
    }

    pub fn push_param(mut self, key: impl Into<Cow<'a, str>>, value: impl Into<PropertyValue<'a>>) -> Self {
        self.inner.params.push(Property::new(key, value));
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

impl<'a> fmt::Display for Method<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(privacy) = self.privacy {
            write!(f, "{privacy} ")?;
        }

        self.inner.fmt(f)
    }
}
