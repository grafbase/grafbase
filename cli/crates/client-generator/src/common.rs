use std::{borrow::Cow, fmt};

#[derive(Debug, Clone)]
pub struct Quoted {
    inner: Cow<'static, str>,
}

impl Quoted {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self { inner: name.into() }
    }
}

impl fmt::Display for Quoted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}'", self.inner)
    }
}

#[derive(Debug, Clone)]
pub struct Template {
    inner: Cow<'static, str>,
}

impl Template {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self { inner: name.into() }
    }
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`", self.inner)
    }
}

#[derive(Debug, Clone)]
pub struct Identifier {
    inner: Cow<'static, str>,
}

impl Identifier {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self { inner: name.into() }
    }
}

impl From<Cow<'static, str>> for Identifier {
    fn from(value: Cow<'static, str>) -> Self {
        Self { inner: value }
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Self {
            inner: Cow::Owned(value),
        }
    }
}

impl From<&'static str> for Identifier {
    fn from(value: &'static str) -> Self {
        Self {
            inner: Cow::Borrowed(value),
        }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}
