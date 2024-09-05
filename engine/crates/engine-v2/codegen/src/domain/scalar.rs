use super::{Definition, Indexed};

#[derive(Debug)]
pub struct Scalar {
    pub indexed: Option<Indexed>,
    pub name: String,
    pub struct_name: String,
    pub has_custom_reader: bool,
    pub copy: bool,
}

impl From<Scalar> for Definition {
    fn from(scalar: Scalar) -> Self {
        Definition::Scalar(scalar)
    }
}

impl Scalar {
    pub fn reader_name(&self) -> &str {
        match self.name.as_str() {
            "String" => "str",
            s => s,
        }
    }
}
