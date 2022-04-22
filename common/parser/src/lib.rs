use async_graphql::registry::Registry;
use async_graphql_parser::{parse_schema, Error as ParserError};
use quick_error::quick_error;
use rules::basic_type::BasicType;
use rules::check_type_validity::CheckTypeValidity;
use rules::check_types_underscore::CheckBeginsWithDoubleUnderscore;
use rules::enum_type::EnumType;
use rules::model_directive::ModelDirective;
use rules::visitor::{visit, RuleError, Visitor, VisitorContext};

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
    let mut rules = rules::visitor::VisitorNil
        .with(ModelDirective)
        .with(CheckBeginsWithDoubleUnderscore)
        .with(BasicType)
        .with(EnumType)
        .with(CheckTypeValidity);

    let schema = parse_schema(format!("{}\n{}", rules.directives(), input.as_ref()))?;

    let mut ctx = VisitorContext::new(&schema);
    visit(&mut rules, &mut ctx, &schema);

    if !ctx.errors.is_empty() {
        return Err(ctx.errors.into());
    }

    let reg = ctx.finish();
    Ok(reg)
}

#[cfg(test)]
mod tests {
    use async_graphql::Schema;
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

        let reg_string = serde_json::to_value(&result).unwrap().to_string();
        let sdl = Schema::new(result).sdl();

        insta::assert_snapshot!(reg_string);
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

        let reg_string = serde_json::to_value(&result).unwrap().to_string();
        let sdl = Schema::new(result).sdl();

        insta::assert_snapshot!(reg_string);
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

        let reg_string = serde_json::to_value(&result).unwrap().to_string();
        let sdl = Schema::new(result).sdl();

        insta::assert_snapshot!(reg_string);
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

        let reg_string = serde_json::to_value(&result).unwrap().to_string();
        let sdl = Schema::new(result).sdl();

        insta::assert_snapshot!(reg_string);
        insta::assert_snapshot!(sdl);
    }
}
