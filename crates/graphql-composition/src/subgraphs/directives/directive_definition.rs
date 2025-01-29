use super::*;
use graphql_federated_graph::DirectiveLocations;
use std::fmt::Display;

pub(crate) struct DirectiveDefinition {
    pub(crate) subgraph_id: SubgraphId,
    pub(crate) name: StringId,
    pub(crate) locations: DirectiveLocations,
    pub(crate) arguments: Vec<InputValueDefinition>,
    pub(crate) repeatable: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct InputValueDefinition {
    pub(crate) name: StringId,
    pub(crate) r#type: FieldTypeId,
    pub(crate) default_value: Option<Value>,
    pub(crate) directives: Vec<Directive>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Directive {
    pub(crate) name: StringId,
    pub(crate) arguments: Vec<(StringId, Value)>,
}

impl Display for Walker<'_, InputValueDefinition> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.subgraphs.walk(self.id.name).as_str())?;
        f.write_str(": ")?;
        Display::fmt(&self.subgraphs.walk(self.id.r#type), f)?;

        if let Some(default) = &self.id.default_value {
            f.write_str(" = ")?;
            Display::fmt(&self.subgraphs.walk(default), f)?;
        }

        for directive in &self.id.directives {
            f.write_str(" ")?;
            Display::fmt(&self.subgraphs.walk(directive), f)?;
        }

        Ok(())
    }
}

impl Display for Walker<'_, &'_ Value> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let subgraphs = self.subgraphs;
        match self.id {
            Value::String(string_id) => {
                graphql_federated_graph::display_graphql_string_literal(subgraphs.walk(*string_id).as_str(), f)
            }
            Value::Int(int) => Display::fmt(int, f),
            Value::Float(float) => Display::fmt(float, f),
            Value::Boolean(b) => Display::fmt(b, f),
            Value::Enum(string_id) => Display::fmt(subgraphs.walk(*string_id).as_str(), f),
            Value::Object(vec) => {
                f.write_str("{")?;
                for (key, value) in vec {
                    Display::fmt(subgraphs.walk(*key).as_str(), f)?;
                    f.write_str(": ")?;
                    Display::fmt(&subgraphs.walk(value), f)?;
                }
                f.write_str("}")
            }
            Value::List(vec) => {
                f.write_str("[")?;
                for value in vec {
                    Display::fmt(&subgraphs.walk(value), f)?;
                }
                f.write_str("]")
            }
            Value::Null => f.write_str("null"),
        }
    }
}

impl Display for Walker<'_, &'_ Directive> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("@")?;
        f.write_str(self.subgraphs.walk(self.id.name).as_str())?;

        if self.id.arguments.is_empty() {
            return Ok(());
        }

        f.write_str("(")?;

        for argument in &self.id.arguments {
            f.write_str(self.subgraphs.walk(argument.0).as_str())?;
            f.write_str(": ")?;
            Display::fmt(&self.subgraphs.walk(&argument.1), f)?;
        }

        f.write_str(")")
    }
}
