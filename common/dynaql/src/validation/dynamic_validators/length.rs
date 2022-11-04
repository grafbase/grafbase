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

fn check_bounds<T: PartialOrd>(item: T, lower: Option<T>, upper: Option<T>) -> LengthTestResult {
    match (
        lower.as_ref().and_then(|lower| item.partial_cmp(lower)),
        upper.as_ref().and_then(|upper| item.partial_cmp(upper)),
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
        let count = match value {
            Value::List(values) => values.len(),
            Value::String(string) => string.chars().count(),
            _ => return,
        };
        match check_bounds(count, self.min, self.max) {
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
