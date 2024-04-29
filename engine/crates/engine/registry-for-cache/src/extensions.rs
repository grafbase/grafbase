//! Extensions to the generated types

use crate::MetaType;

impl<'a> MetaType<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            MetaType::Object(object) => object.name(),
            MetaType::Interface(iface) => iface.name(),
        }
    }
}
