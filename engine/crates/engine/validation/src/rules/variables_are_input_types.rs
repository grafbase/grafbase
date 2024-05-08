use {
    super::concrete_type_name_from_parsed_type,
    crate::visitor::{Visitor, VisitorContext},
    engine_parser::{types::VariableDefinition, Positioned},
};

#[derive(Default)]
pub struct VariablesAreInputTypes;

impl<'a> Visitor<'a, registry_v2::Registry> for VariablesAreInputTypes {
    fn enter_variable_definition(
        &mut self,
        ctx: &mut VisitorContext<'a, registry_v2::Registry>,
        variable_definition: &'a Positioned<VariableDefinition>,
    ) {
        let name = concrete_type_name_from_parsed_type(&variable_definition.node.var_type.node);
        if let Some(ty) = ctx.registry.lookup_type(name) {
            if !ty.is_input() {
                ctx.report_error(
                    vec![variable_definition.pos],
                    format!(
                        "Variable \"{}\" cannot be of non-input type \"{}\"",
                        variable_definition.node.name.node,
                        ty.name()
                    ),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory() -> VariablesAreInputTypes {
        VariablesAreInputTypes
    }

    #[test]
    fn input_types_are_valid() {
        expect_passes_rule!(
            factory,
            r"
          query Foo($a: String, $b: [Boolean!]!, $c: ComplexInput) {
            field(a: $a, b: $b, c: $c)
          }
        ",
        );
    }

    #[test]
    fn output_types_are_invalid() {
        expect_fails_rule!(
            factory,
            r"
          query Foo($a: Dog, $b: [[CatOrDog!]]!, $c: Pet) {
            field(a: $a, b: $b, c: $c)
          }
        ",
        );
    }
}
