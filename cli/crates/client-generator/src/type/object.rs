use std::fmt;

use super::Property;

#[derive(Debug, Default, Clone)]
pub struct ObjectTypeDef {
    properties: Vec<Property>,
    multiline: bool,
}

impl ObjectTypeDef {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    pub fn push_property(&mut self, prop: Property) {
        self.properties.push(prop);
    }
}

impl fmt::Display for ObjectTypeDef {
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
