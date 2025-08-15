use std::borrow::Cow;

use itertools::Itertools;

#[derive(Default)]
pub(crate) struct Attrs<'a> {
    label: Cow<'a, str>,
    others: Vec<Cow<'a, str>>,
}

impl<'a> Attrs<'a> {
    pub fn label(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            others: vec![],
        }
    }

    pub fn label_if(cond: bool, label: impl Into<Cow<'a, str>>) -> Self {
        if cond {
            Self {
                label: label.into(),
                others: vec![],
            }
        } else {
            Self::default()
        }
    }

    #[must_use]
    pub fn bold(mut self) -> Self {
        if !self.label.is_empty() {
            self.label = Cow::Owned(format!("<<b>{}</b>>", self.label));
        }
        self
    }

    #[must_use]
    pub fn with(mut self, attr: impl Into<Cow<'a, str>>) -> Self {
        let attr: Cow<'a, str> = attr.into();
        if !attr.trim().is_empty() {
            self.others.push(attr);
        }
        self
    }

    #[must_use]
    pub fn with_if(self, cond: bool, attr: impl Into<Cow<'a, str>>) -> Self {
        if cond { self.with(attr) } else { self }
    }
}

impl std::fmt::Display for Attrs<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut has_label = false;
        if self.label.starts_with("<") {
            has_label = true;
            write!(f, "label = {}", self.label,)?;
        } else if !self.label.trim().is_empty() {
            has_label = true;
            write!(f, "label = \"{}\"", self.label,)?;
        }
        if has_label && !self.others.is_empty() {
            write!(f, ", ")?;
        }

        write!(f, "{} ", self.others.iter().join(", "))
    }
}
