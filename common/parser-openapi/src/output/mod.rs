use std::fmt::Write;

use case::CaseExt;

use crate::graph::{OpenApiGraph, WrappingType};

pub fn output(graph: &OpenApiGraph) -> Result<String, std::fmt::Error> {
    let mut buffer = String::new();
    for output_type in graph.output_types() {
        let Some(name) = output_type.name(graph) else { continue; };

        writeln!(&mut buffer, "type {name} {{")?;
        for field in output_type.fields(graph) {
            writeln!(&mut buffer, "    {field}")?;
        }
        writeln!(&mut buffer, "}}\n")?;
    }

    let query_operations = graph.query_operations();
    if !query_operations.is_empty() {
        writeln!(&mut buffer, "extend type Query {{")?;
        for op in query_operations {
            let Some(name) = op.name(graph) else { continue; };
            let Some(ty) = op.ty(graph) else { continue; };

            writeln!(&mut buffer, "    {name}: {ty}")?;
        }
        writeln!(&mut buffer, "}}")?;
    }

    Ok(buffer)
}

pub struct Field {
    pub api_name: String,
    pub ty: FieldType,
}

impl Field {
    pub fn new(api_name: String, ty: FieldType) -> Self {
        Field { api_name, ty }
    }

    pub fn graphql_name(&self) -> String {
        self.api_name.to_camel_lowercase()
    }
}

pub enum FieldType {
    Required(Box<FieldType>),
    List(Box<FieldType>),
    Named(String),
}

impl FieldType {
    pub fn new(wrapping: &WrappingType, name: String) -> FieldType {
        match wrapping {
            WrappingType::Required(inner) => FieldType::Required(Box::new(FieldType::new(inner.as_ref(), name))),
            WrappingType::List(inner) => FieldType::List(Box::new(FieldType::new(inner.as_ref(), name))),
            WrappingType::Named => FieldType::Named(name),
        }
    }
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.graphql_name(), self.ty)
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::Required(inner) => write!(f, "{inner}!"),
            FieldType::List(inner) => write!(f, "[{inner}]"),
            FieldType::Named(name) => write!(f, "{name}"),
        }
    }
}
