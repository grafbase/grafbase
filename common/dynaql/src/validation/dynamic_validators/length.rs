use dynaql_value::Value;
use std::cmp::Ordering;

use crate::validation::visitor::VisitorContext;

use super::DynValidate;
use crate::Pos;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LengthValidator {
    min: Option<usize>,
    max: Option<usize>,
}

impl LengthValidator {
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        LengthValidator { min, max }
    }
}

enum LengthTestResult {
    TooShort,
    TooLong,
    InBounds,
}

fn check_length(count: usize, min: Option<usize>, max: Option<usize>) -> LengthTestResult {
    match (
        min.as_ref().and_then(|min| count.partial_cmp(min)),
        max.as_ref().and_then(|max| count.partial_cmp(max)),
    ) {
        (Some(Ordering::Less), _) => LengthTestResult::TooShort,
        (_, Some(Ordering::Greater)) => LengthTestResult::TooLong,
        (
            None | Some(Ordering::Greater | Ordering::Equal),
            None | Some(Ordering::Less | Ordering::Equal),
        ) => LengthTestResult::InBounds,
    }
}

impl DynValidate<&Value> for LengthValidator {
    fn validate<'a>(&self, ctx: &mut VisitorContext<'a>, pos: Pos, value: &Value) {
        use LengthTestResult::*;
        match value {
            Value::List(values) => {
                let count = values.len();
                match check_length(count, self.min, self.max) {
                    InBounds => (),
                    TooLong => ctx.report_error(
                        vec![pos],
                        "{count} is too long, must be shorter than {max}".to_string(),
                    ),
                    TooShort => ctx.report_error(
                        vec![pos],
                        "{count} is too short, must be at least {min} long".to_string(),
                    ),
                }
            }
            Value::String(string) => {
                let count = string.chars().count();
                match check_length(count, self.min, self.max) {
                    InBounds => (),
                    TooLong => ctx.report_error(
                        vec![pos],
                        "{count} is too long, must be shorter than {max}".to_string(),
                    ),
                    TooShort => ctx.report_error(
                        vec![pos],
                        "{count} is too short, must be at least {min} long".to_string(),
                    ),
                }
            }
            _ => (),
        }
    }
}

#[test]
fn test_length_validator() {
    use super::DynValidator;
    use crate::parser::parse_query;
    use crate::{EmptyMutation, EmptySubscription, Object, Schema};

    struct Query;

    #[Object(internal)]
    #[allow(unreachable_code)]
    impl Query {
        async fn value(&self) -> i32 {
            todo!()
        }
    }

    let registry = Schema::create_registry_static::<Query, EmptyMutation, EmptySubscription>();
    let query = r#"{
        value #1
    }"#;

    let doc = parse_query(query).unwrap();

    let mut ctx = VisitorContext::new(&registry, &doc, None);
    let custom_validator = DynValidator::length(Some(0), None);
    custom_validator.validate(
        &mut ctx,
        Pos::from((0, 0)),
        &Value::String("test".to_string()),
    );
    assert!(ctx.errors.is_empty());

    let custom_validator = DynValidator::length(Some(0), Some(1));
    custom_validator.validate(
        &mut ctx,
        Pos::from((0, 0)),
        &Value::String("test".to_string()),
    );
    assert_eq!(ctx.errors.len(), 1)
}
