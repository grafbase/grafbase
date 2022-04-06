use async_graphql::registry::Registry;


use async_graphql_parser::{parse_schema, Error as ParserError};
use quick_error::quick_error;

mod rules;
use rules::model_directive::ModelDirective;
use rules::visitor::{visit, RuleError, Visitor, VisitorContext};
mod registry;
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
    let mut rules = rules::visitor::VisitorNil.with(ModelDirective);

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
    use async_graphql::{Schema};
    
    use serde_json as _;

    #[test]
    fn test_simple() {
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
}
