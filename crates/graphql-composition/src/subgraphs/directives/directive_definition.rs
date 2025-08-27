use super::*;
use crate::federated_graph::{DirectiveLocations, display_graphql_string_literal};
use std::fmt::{self, Display};

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
    pub(crate) r#type: FieldType,
    pub(crate) default_value: Option<Value>,
    pub(crate) directives: Vec<InputValueDefinitionDirective>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct InputValueDefinitionDirective {
    pub(crate) name: StringId,
    pub(crate) arguments: Vec<(StringId, Value)>,
}

impl InputValueDefinition {
    pub(crate) fn display<'a>(&'a self, subgraphs: &'a Subgraphs) -> impl fmt::Display + 'a {
        struct DisplayImpl<'a>(&'a InputValueDefinition, &'a Subgraphs);

        impl fmt::Display for DisplayImpl<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let DisplayImpl(input_value_definition, subgraphs) = self;

                f.write_str(&subgraphs[input_value_definition.name])?;
                f.write_str(": ")?;
                Display::fmt(&input_value_definition.r#type.display(subgraphs), f)?;

                if let Some(default) = &input_value_definition.default_value {
                    f.write_str(" = ")?;
                    Display::fmt(&default.display(subgraphs), f)?;
                }

                for directive in &input_value_definition.directives {
                    f.write_str(" ")?;
                    Display::fmt(&directive.display(subgraphs), f)?;
                }

                Ok(())
            }
        }

        DisplayImpl(self, subgraphs)
    }
}

impl Value {
    pub(crate) fn display<'a>(&'a self, subgraphs: &'a Subgraphs) -> impl fmt::Display + 'a {
        struct DisplayImpl<'a>(&'a Value, &'a Subgraphs);

        impl fmt::Display for DisplayImpl<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let DisplayImpl(value, subgraphs) = self;

                match value {
                    Value::String(string_id) => display_graphql_string_literal(&subgraphs[*string_id], f),
                    Value::Int(int) => Display::fmt(int, f),
                    Value::Float(float) => Display::fmt(float, f),
                    Value::Boolean(b) => Display::fmt(b, f),
                    Value::Enum(string_id) => Display::fmt(&subgraphs[*string_id], f),
                    Value::Object(vec) => {
                        f.write_str("{")?;
                        for (key, value) in vec {
                            Display::fmt(&subgraphs[*key], f)?;
                            f.write_str(": ")?;
                            Display::fmt(&value.display(subgraphs), f)?;
                        }
                        f.write_str("}")
                    }
                    Value::List(vec) => {
                        f.write_str("[")?;
                        for value in vec {
                            Display::fmt(&value.display(subgraphs), f)?;
                        }
                        f.write_str("]")
                    }
                    Value::Null => f.write_str("null"),
                }
            }
        }

        DisplayImpl(self, subgraphs)
    }
}

impl InputValueDefinitionDirective {
    pub(crate) fn display<'a>(&'a self, subgraphs: &'a Subgraphs) -> impl fmt::Display + 'a {
        struct DisplayImpl<'a>(&'a InputValueDefinitionDirective, &'a Subgraphs);

        impl fmt::Display for DisplayImpl<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let DisplayImpl(directive, subgraphs) = self;

                f.write_str("@")?;
                f.write_str(&subgraphs[directive.name])?;

                if directive.arguments.is_empty() {
                    return Ok(());
                }

                f.write_str("(")?;

                for argument in &directive.arguments {
                    f.write_str(&subgraphs[argument.0])?;
                    f.write_str(": ")?;
                    argument.1.display(subgraphs).fmt(f)?;
                }

                f.write_str(")")
            }
        }

        DisplayImpl(self, subgraphs)
    }
}
