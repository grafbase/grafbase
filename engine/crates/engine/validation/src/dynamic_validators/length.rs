use std::cmp::Ordering;

use engine_parser::Pos;
use engine_value::{ConstValue, Value};
use registry_v2::{validators::LengthValidator, MetaInputValue};

use super::DynValidate;
use crate::visitor::VisitorContext;

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
        (None | Some(Ordering::Greater | Ordering::Equal), None | Some(Ordering::Less | Ordering::Equal)) => {
            LengthTestResult::InBounds
        }
    }
}

impl DynValidate<&Value> for LengthValidator {
    fn validate(
        &self,
        ctx: &mut VisitorContext<'_, registry_v2::Registry>,
        meta: MetaInputValue<'_>,
        pos: Pos,
        value: &Value,
    ) {
        use LengthTestResult::*;

        let var_value = match value {
            Value::Variable(var_name) => ctx
                .variables
                .and_then(|variables| variables.get(var_name).cloned().map(ConstValue::into_value)),
            _ => None,
        };
        let count = match var_value.as_ref().unwrap_or(value) {
            Value::List(values) => values.len(),
            Value::String(string) => string.chars().count(),
            _ => return,
        };
        let name = meta.name();
        match check_bounds(count, self.min, self.max) {
            InBounds => (),
            TooLong => ctx.report_error(
                vec![pos],
                format!(
                    "Invalid value for argument \"{name}\", length {count} is too long, must be no larger than {}",
                    self.max.expect("max must have been some for this case to be hit")
                ),
            ),
            TooShort => ctx.report_error(
                vec![pos],
                format!(
                    "Invalid value for argument \"{name}\", length {count} is too short, must be at least {}",
                    self.min.expect("min must have been some for this case to be hit")
                ),
            ),
        }
    }
}
