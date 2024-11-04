use super::{Definition, Indexed};

#[derive(Clone, Debug)]
pub enum Scalar {
    Value {
        indexed: Option<Indexed>,
        name: String,
        span: cynic_parser::Span,
        external_domain_name: Option<String>,
        in_prelude: bool,
        copy: bool,
    },
    Record {
        indexed: Option<Indexed>,
        name: String,
        span: cynic_parser::Span,
        record_name: String,
        external_domain_name: Option<String>,
        in_prelude: bool,
        copy: bool,
    },
    Ref {
        name: String,
        id_struct_name: String,
        span: cynic_parser::Span,
        in_prelude: bool,
        external_domain_name: Option<String>,
        target: Box<Definition>,
    },
}

// #[derive(Debug)]
// pub struct Scalar {
//     pub indexed: Option<Indexed>,
//     pub span: cynic_parser::Span,
//     pub name: String,
//     pub struct_name: String,
//     pub walker_name: String,
//     pub is_record: bool,
//     pub copy: bool,
//     pub external_domain_name: Option<String>,
//     pub in_prelude: bool,
// }

impl From<Scalar> for Definition {
    fn from(scalar: Scalar) -> Self {
        Definition::Scalar(scalar)
    }
}

impl Scalar {
    pub fn name(&self) -> &str {
        match self {
            Scalar::Value { name, .. } => name,
            Scalar::Record { name, .. } => name,
            Scalar::Ref { name, .. } => name,
        }
    }
    pub fn span(&self) -> &cynic_parser::Span {
        match self {
            Scalar::Value { span, .. } => span,
            Scalar::Record { span, .. } => span,
            Scalar::Ref { span, .. } => span,
        }
    }

    pub fn external_domain_name(&self) -> Option<&str> {
        match self {
            Scalar::Value {
                external_domain_name, ..
            } => external_domain_name.as_deref(),
            Scalar::Record {
                external_domain_name, ..
            } => external_domain_name.as_deref(),
            Scalar::Ref {
                external_domain_name, ..
            } => external_domain_name.as_deref(),
        }
    }

    pub fn walker_name(&self) -> &str {
        match self {
            Scalar::Value { name, .. } => match name.as_str() {
                "String" => "str",
                s => s,
            },
            Scalar::Record { name, .. } => name,
            Scalar::Ref { target, .. } => target.walker_name(),
        }
    }
}
