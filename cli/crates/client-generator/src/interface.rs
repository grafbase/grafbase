use std::{borrow::Cow, fmt};

use crate::r#type::{Property, StaticType};

#[derive(Debug)]
pub struct Interface {
    identifier: StaticType,
    properties: Vec<Property>,
}

impl Interface {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            identifier: StaticType::ident(name),
            properties: Vec::new(),
        }
    }

    pub fn push_property(&mut self, prop: Property) {
        self.properties.push(prop);
    }
}

impl fmt::Display for Interface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "interface {} {{", self.identifier)?;

        for prop in &self.properties {
            writeln!(f, "  {prop};")?;
        }

        f.write_str("};")?;

        Ok(())
    }
}
