//! ### What it does
//!
//! Check that relation names only contain valid characters.
//!
//! ### Why?
//!
//! There are limitations on these and if we don't validate here
//! users get runtime errors

use super::relations_engine::{NAME_ARGUMENT, RELATION_DIRECTIVE};
use crate::rules::visitor::{Visitor, VisitorContext};
use dynaql_value::ConstValue;
use if_chain::if_chain;
use regex::Regex;

pub struct CheckRelationName;

static NAME_CHARS: &str = r"[_a-zA-Z0-9]";

lazy_static::lazy_static! {
    static ref NAME_RE: Regex = Regex::new(&format!("^{NAME_CHARS}*$")).unwrap();
}

impl<'a> Visitor<'a> for CheckRelationName {
    fn enter_directive(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        directive: &'a dynaql::Positioned<dynaql_parser::types::ConstDirective>,
    ) {
        if_chain! {
            let directive = &directive.node;
            if directive.name.node == RELATION_DIRECTIVE;
            if let Some(value) = directive.get_argument(NAME_ARGUMENT);
            if let ConstValue::String(name) = &value.node;
            if !NAME_RE.is_match(name);
            then {
                ctx.report_error(
                    vec![value.pos],
                    format!("Relation names should only contain {NAME_CHARS} but {name} does not"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CheckRelationName;
    use crate::rules::visitor::{visit, VisitorContext};
    use dynaql_parser::parse_schema;

    #[test]
    fn should_error_on_invalid_name() {
        let schema = r#"
            type Todo @model {
                secondaryAuthors: [Author] @relation(name: "second-author")
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut CheckRelationName, &mut ctx, &schema);

        assert!(!ctx.errors.is_empty(), "shouldn't be empty");
        assert_eq!(ctx.errors.len(), 1, "should have one error");
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "Relation names should only contain [_a-zA-Z0-9] but second-author does not",
        );
    }

    #[test]
    fn should_allow_valid_names() {
        let schema = r#"
            type Todo @model {
                secondaryAuthors: [Author] @relation(name: "second_author")
                evenMoreAuthors: [Author] @relation(name: "moreAuthors")
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut CheckRelationName, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should not have any error");
    }
}
