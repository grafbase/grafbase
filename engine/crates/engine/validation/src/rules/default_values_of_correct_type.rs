use engine_parser::{types::BaseType, Positioned};
use query_path::QueryPath;

use {
    crate::{
        utils::is_valid_input_value,
        visitor::{Visitor, VisitorContext},
    },
    engine_parser::types::VariableDefinition,
};

pub struct DefaultValuesOfCorrectType;

impl<'a> Visitor<'a> for DefaultValuesOfCorrectType {
    fn enter_variable_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        variable_definition: &'a Positioned<VariableDefinition>,
    ) {
        if let BaseType::Named(vtype_name) = &variable_definition.node.var_type.node.base {
            if ctx.registry.lookup_type(vtype_name.as_str()).is_none() {
                ctx.report_error(vec![variable_definition.pos], format!(r#"Unknown type "{vtype_name}""#));
                return;
            }
        }

        if let Some(value) = &variable_definition.node.default_value {
            if !variable_definition.node.var_type.node.nullable {
                ctx.report_error(
                    vec![variable_definition.pos],
                    format!(
                        "Argument \"{}\" has type \"{}\" and is not nullable, so it can't have a default value",
                        variable_definition.node.name, variable_definition.node.var_type,
                    ),
                );
            } else if let Some(reason) = is_valid_input_value(
                ctx.registry,
                &variable_definition.node.var_type.to_string(),
                &value.node,
                QueryPath::empty().child(variable_definition.node.name.node.as_str()),
            ) {
                ctx.report_error(
                    vec![variable_definition.pos],
                    format!("Invalid default value for argument: {reason}"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory() -> DefaultValuesOfCorrectType {
        DefaultValuesOfCorrectType
    }

    #[test]
    fn variables_with_no_default_values() {
        expect_passes_rule!(
            factory,
            r"
          query NullableValues($a: Int, $b: String, $c: ComplexInput) {
            dog { name }
          }
        ",
        );
    }

    #[test]
    fn required_variables_without_default_values() {
        expect_passes_rule!(
            factory,
            r"
          query RequiredValues($a: Int!, $b: String!) {
            dog { name }
          }
        ",
        );
    }

    #[test]
    fn variables_with_valid_default_values() {
        expect_passes_rule!(
            factory,
            r#"
          query WithDefaultValues(
            $a: Int = 1,
            $b: String = "ok",
            $c: ComplexInput = { requiredField: true, intField: 3 }
          ) {
            dog { name }
          }
        "#,
        );
    }

    #[test]
    fn no_required_variables_with_default_values() {
        expect_fails_rule!(
            factory,
            r#"
          query UnreachableDefaultValues($a: Int! = 3, $b: String! = "default") {
            dog { name }
          }
        "#,
        );
    }

    #[test]
    fn variables_with_invalid_default_values() {
        expect_fails_rule!(
            factory,
            r#"
          query InvalidDefaultValues(
            $a: Int = "one",
            $b: String = 4,
            $c: ComplexInput = "notverycomplex"
          ) {
            dog { name }
          }
        "#,
        );
    }

    #[test]
    fn complex_variables_missing_required_field() {
        expect_fails_rule!(
            factory,
            r"
          query MissingRequiredField($a: ComplexInput = {intField: 3}) {
            dog { name }
          }
        ",
        );
    }

    #[test]
    fn list_variables_with_invalid_item() {
        expect_fails_rule!(
            factory,
            r#"
          query InvalidItem($a: [String] = ["one", 2]) {
            dog { name }
          }
        "#,
        );
    }
}
