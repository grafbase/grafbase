use cynic_parser::common::WrappingType;

use super::{Definition, Indexed, Meta};

#[derive(Debug, Clone)]
pub struct Object {
    pub meta: Meta,
    pub span: cynic_parser::Span,
    pub description: Option<String>,
    pub indexed: Option<Indexed>,
    pub name: String,
    pub struct_name: String,
    pub copy: bool,
    pub fields: Vec<Field>,
    pub external_domain_name: Option<String>,
}

impl From<Object> for Definition {
    fn from(object: Object) -> Self {
        Definition::Object(object)
    }
}

impl Object {
    pub fn walker_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub record_field_name: String,
    pub type_name: String,
    /// The wrapper types from the outermost to innermost
    pub wrapping: Vec<WrappingType>,

    /// If set, the field should be represented as a Vec<Id> rather than an IdRange<Id>
    pub vec: bool,
}

impl Field {
    pub fn has_list_wrapping(&self) -> bool {
        self.wrapping.iter().any(|w| matches!(w, WrappingType::List))
    }
}
