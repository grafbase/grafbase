use dynaql_value::Value;

use crate::validation::visitor::VisitorContext;
use crate::Pos;

use std::sync::Arc;

mod length;

use length::LengthValidator;

pub(crate) trait DynValidate<T> {
    fn validate<'a>(&self, _ctx: &mut VisitorContext<'a>, pos: Pos, other: T);
}

// Wrap Validators up in an enum to avoid having to box the context data
#[derive(Clone, derivative::Derivative)]
pub enum DynValidator {
    Length(LengthValidator),
    Custom(Arc<dyn Fn(&Value) -> Result<(), String> + Send + Sync>),
}

impl DynValidator {
    pub fn from_fn<F>(func: F) -> Self
    where
        F: Fn(&Value) -> Result<(), String> + Send + Sync + 'static,
    {
        Self::Custom(Arc::new(func))
    }

    pub fn length(min: Option<usize>, max: Option<usize>) -> Self {
        Self::Length(LengthValidator::new(min, max))
    }
}

impl DynValidator {
    fn inner(&self) -> &dyn DynValidate<&Value> {
        use DynValidator::*;
        match self {
            Length(v) => v,
            Custom(f) => f,
        }
    }
}

impl DynValidate<&Value> for DynValidator {
    fn validate<'a>(&self, ctx: &mut VisitorContext<'a>, pos: Pos, value: &Value) {
        self.inner().validate(ctx, pos, value)
    }
}

impl DynValidate<&Value> for Arc<dyn Fn(&Value) -> Result<(), String> + Send + Sync> {
    fn validate<'a>(&self, ctx: &mut VisitorContext<'a>, pos: Pos, value: &Value) {
        if let Err(message) = self(value) {
            ctx.report_error(vec![pos], message);
        };
    }
}

#[test]
fn test_custom() {
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
    let custom_validator = DynValidator::from_fn(|_| Ok(()));
    custom_validator.validate(
        &mut ctx,
        Pos::from((0, 0)),
        &Value::String("test".to_string()),
    );
    assert!(ctx.errors.is_empty());

    let custom_validator = DynValidator::from_fn(|_| Err("Error!".to_string()));
    custom_validator.validate(
        &mut ctx,
        Pos::from((0, 0)),
        &Value::String("test".to_string()),
    );
    assert_eq!(ctx.errors.len(), 1)
}
