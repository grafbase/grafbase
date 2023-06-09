use std::{borrow::Cow, fmt};

use crate::common::{Identifier, Quoted};

#[derive(Debug)]
pub struct Import {
    items: ImportItems,
    import_location: Quoted,
}

impl Import {
    pub fn all_as(import_location: impl Into<Cow<'static, str>>, alias: impl Into<Cow<'static, str>>) -> Self {
        Self {
            import_location: Quoted::new(import_location),
            items: ImportItems::All { alias: alias.into() },
        }
    }

    pub fn items(import_location: impl Into<Cow<'static, str>>, items: &[&'static str]) -> Self {
        Self {
            import_location: Quoted::new(import_location),
            items: ImportItems::Set(items.iter().map(|i| Identifier::new(*i)).collect()),
        }
    }

    pub fn push_item(&mut self, identifier: Identifier) {
        match self.items {
            ImportItems::All { .. } => self.items = ImportItems::Set(vec![identifier]),
            ImportItems::Set(ref mut items) => items.push(identifier),
        }
    }
}

impl fmt::Display for Import {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "import {} from {}", self.items, self.import_location)
    }
}

#[derive(Debug)]
pub enum ImportItems {
    All { alias: Cow<'static, str> },
    Set(Vec<Identifier>),
}

impl fmt::Display for ImportItems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportItems::All { alias } => write!(f, "* as {alias}"),
            ImportItems::Set(ref identifiers) => {
                if identifiers.len() > 1 {
                    f.write_str("{ ")?;
                }

                for (i, ident) in identifiers.iter().enumerate() {
                    ident.fmt(f)?;

                    if i < identifiers.len() - 1 {
                        f.write_str(", ")?;
                    }
                }

                if identifiers.len() > 1 {
                    f.write_str(" }")?;
                }

                Ok(())
            }
        }
    }
}
