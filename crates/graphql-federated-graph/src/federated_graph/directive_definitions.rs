use super::*;

#[derive(Debug, Clone)]
pub struct DirectiveDefinition {
    pub namespace: Option<StringId>,
    pub name: StringId,
    pub locations: DirectiveLocations,
    pub arguments: InputValueDefinitions,
    pub repeatable: bool,
}

bitflags::bitflags! {
    /// https://spec.graphql.org/October2021/#sec-The-__Directive-Type
    #[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
    pub struct DirectiveLocations: u32 {
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

impl fmt::Display for DirectiveLocations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
