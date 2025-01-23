use super::*;
use bitflags::bitflags;
use std::fmt::Display;

pub(crate) struct DirectiveDefinition {
    pub(crate) subgraph_id: SubgraphId,
    pub(crate) name: StringId,
    pub(crate) locations: DirectiveLocations,
    pub(crate) arguments: Vec<InputValueDefinition>,
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

bitflags! {
    /// https://spec.graphql.org/October2021/#sec-The-__Directive-Type
    #[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
    pub(crate) struct DirectiveLocations: u32 {
        const QUERY = 0b1 << 0;
        const MUTATION = 0b1 << 1;
        const SUBSCRIPTION = 0b1 << 2;
        const FIELD = 0b1 << 3;
        const FRAGMENT_DEFINITION = 0b1 << 4;
        const FRAGMENT_SPREAD = 0b1 << 5;
        const INLINE_FRAGMENT = 0b1 << 6;
        const VARIABLE_DEFINITION = 0b1 << 7;
        const SCHEMA = 0b1 << 8;
        const SCALAR = 0b1 << 9;
        const OBJECT = 0b1 << 10;
        const FIELD_DEFINITION = 0b1 << 11;
        const ARGUMENT_DEFINITION = 0b1 << 12;
        const INTERFACE = 0b1 << 13;
        const UNION = 0b1 << 14;
        const ENUM = 0b1 << 15;
        const ENUM_VALUE = 0b1 << 16;
        const INPUT_OBJECT = 0b1 << 17;
        const INPUT_FIELD_DEFINITION = 0b1 << 18;
    }
}

impl Display for DirectiveLocations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut locations = self.iter().peekable();

        while let Some(location) = locations.next() {
            let name = match location {
                DirectiveLocations::QUERY => "QUERY",
                DirectiveLocations::MUTATION => "MUTATION",
                DirectiveLocations::SUBSCRIPTION => "SUBSCRIPTION",
                DirectiveLocations::FIELD => "FIELD",
                DirectiveLocations::FRAGMENT_DEFINITION => "FRAGMENT_DEFINITION",
                DirectiveLocations::FRAGMENT_SPREAD => "FRAGMENT_SPREAD",
                DirectiveLocations::INLINE_FRAGMENT => "INLINE_FRAGMENT",
                DirectiveLocations::VARIABLE_DEFINITION => "VARIABLE_DEFINITION",
                DirectiveLocations::SCHEMA => "SCHEMA",
                DirectiveLocations::SCALAR => "SCALAR",
                DirectiveLocations::OBJECT => "OBJECT",
                DirectiveLocations::FIELD_DEFINITION => "FIELD_DEFINITION",
                DirectiveLocations::ARGUMENT_DEFINITION => "ARGUMENT_DEFINITION",
                DirectiveLocations::INTERFACE => "INTERFACE",
                DirectiveLocations::UNION => "UNION",
                DirectiveLocations::ENUM => "ENUM",
                DirectiveLocations::ENUM_VALUE => "ENUM_VALUE",
                DirectiveLocations::INPUT_OBJECT => "INPUT_OBJECT",
                DirectiveLocations::INPUT_FIELD_DEFINITION => "INPUT_FIELD_DEFINITION",
                _ => unreachable!(),
            };

            f.write_str(name)?;

            if locations.peek().is_some() {
                f.write_str(" | ")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directive_definitions_display() {
        let all = DirectiveLocations::all().to_string();

        let expected = "QUERY | MUTATION | SUBSCRIPTION | FIELD | FRAGMENT_DEFINITION | FRAGMENT_SPREAD | INLINE_FRAGMENT | VARIABLE_DEFINITION | SCHEMA | SCALAR | OBJECT | FIELD_DEFINITION | ARGUMENT_DEFINITION | INTERFACE | UNION | ENUM | ENUM_VALUE | INPUT_OBJECT | INPUT_FIELD_DEFINITION";

        assert_eq!(all, expected);
    }
}
