use std::collections::HashMap;

use dynaql::registry::scalars::{PossibleScalar, SDLDefinitionScalar};
use dynaql_parser::{parse_schema, Error as ParserError};
use quick_error::quick_error;
use rules::auth_directive::AuthDirective;
use rules::basic_type::BasicType;
use rules::check_field_lowercase::CheckFieldCamelCase;
use rules::check_field_not_reserved::CheckModelizedFieldReserved;
use rules::check_known_directives::CheckAllDirectivesAreKnown;
use rules::check_type_validity::CheckTypeValidity;
use rules::check_types_underscore::CheckBeginsWithDoubleUnderscore;
use rules::default_directive::DefaultDirective;
use rules::enum_type::EnumType;
use rules::model_directive::ModelDirective;
use rules::relations::relations_rules;
use rules::unique_directive::UniqueDirective;
use rules::visitor::{visit, RuleError, Visitor, VisitorContext};

pub use dynaql::registry::Registry;

use crate::rules::scalar_hydratation::ScalarHydratation;

mod dynamic_string;
mod registry;
mod rules;
mod utils;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Parser(err: ParserError) {
            from()
            source(err)
            display("{}", err)
        }
        Validation(err: Vec<RuleError>) {
            from()
            display("{:?}", err)
        }
    }
}

/// Transform the input schema into a Registry
pub fn to_registry<S: AsRef<str>>(input: S) -> Result<Registry, Error> {
    to_registry_with_variables(input, &HashMap::new())
}

/// Transform the input schema into a Registry in the context of provided environment variables
pub fn to_registry_with_variables<S: AsRef<str>>(
    input: S,
    variables: &HashMap<String, String>,
) -> Result<Registry, Error> {
    let mut rules = rules::visitor::VisitorNil
        .with(CheckBeginsWithDoubleUnderscore)
        .with(CheckModelizedFieldReserved)
        .with(CheckFieldCamelCase)
        .with(CheckTypeValidity)
        .with(DefaultDirective)
        .with(UniqueDirective)
        .with(ModelDirective)
        .with(AuthDirective)
        .with(BasicType)
        .with(EnumType)
        .with(ScalarHydratation)
        .with(relations_rules())
        .with(CheckAllDirectivesAreKnown::default());

    let schema = format!("{}\n{}\n{}", PossibleScalar::sdl(), rules.directives(), input.as_ref());

    let schema = parse_schema(schema)?;

    let mut ctx = VisitorContext::new_with_variables(&schema, variables);
    visit(&mut rules, &mut ctx, &schema);

    if !ctx.errors.is_empty() {
        return Err(ctx.errors.into());
    }

    let reg = ctx.finish();
    Ok(reg)
}

#[cfg(test)]
mod tests {
    use dynaql::Schema;
    use serde_json as _;

    #[test]
    fn test_simple_product() {
        let result = super::to_registry(
            r#"
            type Product @model {
                id: ID!
                name: String!
                """
                The product's price in $
                """
                price: Int!
            }
            "#,
        )
        .unwrap();

        let reg_string = serde_json::to_value(&result).unwrap();
        let sdl = Schema::new(result).sdl();

        insta::assert_json_snapshot!(reg_string);
        insta::assert_snapshot!(sdl);
    }

    #[test]
    fn test_simple_todo() {
        let result = super::to_registry(
            r#"
            type Todo @model {
              id: ID!
              content: String!
              author: Author
            }

            type Author {
              name: String!
              lastname: String!
              pseudo: String
              truc: Truc!
            }

            type Truc {
              name: String!
            }
            "#,
        )
        .unwrap();

        let reg_string = serde_json::to_value(&result).unwrap();
        let sdl = Schema::new(result).sdl();

        insta::assert_json_snapshot!(reg_string);
        insta::assert_snapshot!(sdl);
    }

    #[test]
    fn test_simple_todo_from_template() {
        let result = super::to_registry(
            r#"
            type TodoList @model {
              id: ID!
              title: String!
              todos: [Todo]
            }

            type Todo @model {
              id: ID!
              title: String!
              complete: Boolean
            }
            "#,
        )
        .unwrap();

        let sdl = Schema::new(result).sdl();

        insta::assert_snapshot!(sdl);
    }

    #[test]
    fn test_simple_todo_with_vec() {
        let result = super::to_registry(
            r#"
            type Todo @model {
              id: ID!
              content: String!
              authors: [Author]
            }

            type Author {
              name: String!
              lastname: String!
              pseudo: String
              truc: Truc!
            }

            type Truc {
              name: String!
            }
            "#,
        )
        .unwrap();

        let reg_string = serde_json::to_value(&result).unwrap();
        let sdl = Schema::new(result).sdl();

        insta::assert_json_snapshot!(reg_string);
        insta::assert_snapshot!(sdl);
    }

    #[test]
    fn test_simple_todo_with_enum() {
        let result = super::to_registry(
            r#"
            """
            A TodoType
            """
            enum TodoType {
              TODO1

              """
              A Type 2 for TODO
              """
              TODO2
            }

            type Todo @model {
              id: ID!
              content: String!
              authors: [Author]
              ty: TodoType!
            }

            type Author {
              name: String!
              lastname: String!
              pseudo: String
              truc: Truc!
            }

            type Truc {
              name: String!
            }
            "#,
        )
        .unwrap();

        let reg_string = serde_json::to_value(&result).unwrap();
        let sdl = Schema::new(result).sdl();

        insta::assert_json_snapshot!(reg_string);
        insta::assert_snapshot!(sdl);
    }

    #[test]
    fn test_simple_post_with_relation() {
        let result = super::to_registry(
            r#"
            enum Country {
              FRANCE
              NOT_FRANCE
            }

            type Blog @model {
              id: ID!
              posts: [Post] 
              owner: Author!
            }

            type Post @model {
              id: ID!
              content: String!
              authors: [Author] @relation(name: "published")
            }

            type Author @model {
              id: ID!
              name: String!
              lastname: String!
              country: Country!
              posts: [Post] @relation(name: "published")
            }
            "#,
        )
        .unwrap();

        let reg_string = serde_json::to_value(&result).unwrap();
        let sdl = Schema::new(result).sdl();

        insta::assert_json_snapshot!(reg_string);
        insta::assert_snapshot!(sdl);
    }

    #[test]
    fn should_ensure_lowercase() {
        let result = super::to_registry(
            r#"
            type Blog @model {
              id: ID!
              truc_break: String! @unique
            }
            "#,
        );

        assert!(result.is_err(), "Should error here");
    }
}
