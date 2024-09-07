use super::{Definition, Indexed};

#[derive(Debug)]
pub struct Scalar {
    pub indexed: Option<Indexed>,
    pub span: cynic_parser::Span,
    pub name: String,
    pub struct_name: String,
    pub is_record: bool,
    pub copy: bool,
}

impl From<Scalar> for Definition {
    fn from(scalar: Scalar) -> Self {
        Definition::Scalar(scalar)
    }
}

impl Scalar {
    pub fn walker_name(&self) -> &str {
        match self.name.as_str() {
            "String" => "str",
            s => s,
        }
    }
}
