use std::{borrow::Cow, fmt};

#[derive(Debug, Clone)]
pub struct Quoted<'a> {
    inner: Cow<'a, str>,
}

impl<'a> Quoted<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self { inner: name.into() }
    }
}

impl<'a> fmt::Display for Quoted<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}'", self.inner)
    }
}

#[derive(Debug, Clone)]
pub struct Template<'a> {
    inner: Cow<'a, str>,
}

#[allow(dead_code)]
impl<'a> Template<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self { inner: name.into() }
    }
}

impl<'a> fmt::Display for Template<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}`", self.inner)
    }
}

#[derive(Debug, Clone)]
pub struct Identifier<'a> {
    inner: Cow<'a, str>,
}

impl<'a> Identifier<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self { inner: name.into() }
    }
}

impl<'a> From<Cow<'a, str>> for Identifier<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self { inner: value }
    }
}

impl From<String> for Identifier<'static> {
    fn from(value: String) -> Self {
        Self {
            inner: Cow::Owned(value),
        }
    }
}

impl<'a> From<&'a str> for Identifier<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            inner: Cow::Borrowed(value),
        }
    }
}

impl<'a> fmt::Display for Identifier<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}
