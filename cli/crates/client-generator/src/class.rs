mod construct;
mod constructor;
mod method;
mod privacy;
mod property;

use std::fmt;

pub use construct::Construct;
pub use constructor::Constructor;
pub use method::Method;
pub use privacy::Privacy;
pub use property::ClassProperty;

use crate::r#type::TypeIdentifier;

#[derive(Debug)]
pub struct Class {
    identifier: TypeIdentifier,
    properties: Vec<ClassProperty>,
    constructor: Option<Constructor>,
    methods: Vec<Method>,
}

impl Class {
    #[must_use]
    pub fn new(identifier: TypeIdentifier) -> Self {
        Self {
            identifier,
            properties: Vec::new(),
            constructor: None,
            methods: Vec::new(),
        }
    }

    pub fn set_constructor(&mut self, constructor: Constructor) {
        self.constructor = Some(constructor);
    }

    pub fn push_property(&mut self, property: ClassProperty) {
        self.properties.push(property);
    }

    pub fn push_method(&mut self, method: Method) {
        self.methods.push(method);
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "class {} {{", self.identifier)?;

        for property in &self.properties {
            writeln!(f, "{property}")?;
        }

        if let Some(ref constructor) = self.constructor {
            writeln!(f)?;
            constructor.fmt(f)?;
        }

        for method in &self.methods {
            writeln!(f)?;
            writeln!(f)?;
            method.fmt(f)?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}
