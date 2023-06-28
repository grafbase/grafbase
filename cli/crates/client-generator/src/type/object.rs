use std::fmt;

use super::Property;

#[derive(Default, Clone)]
pub struct ObjectTypeDef<'a> {
    properties: Vec<Property<'a>>,
    multiline: bool,
}

impl<'a> ObjectTypeDef<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    pub fn push_property(&mut self, prop: Property<'a>) {
        self.properties.push(prop);
    }
}

impl<'a> fmt::Display for ObjectTypeDef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let divider = if self.multiline { "\n" } else { " " };
        let indent = if self.multiline { "  " } else { "" };

        write!(f, "{{{divider}")?;

        for (i, prop) in self.properties.iter().enumerate() {
            write!(f, "{indent}{prop}")?;

            if i < self.properties.len() - 1 {
                write!(f, ",{divider}")?;
            }
        }

        write!(f, "{divider}}}")?;

        Ok(())
    }
}
