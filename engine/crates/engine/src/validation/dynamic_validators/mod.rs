use engine_value::Value;
use registry_v2::{validators::DynValidator, MetaInputValue};

use crate::{validation::visitor::VisitorContext, Pos};

mod length;

pub(crate) trait DynValidate<T> {
    fn validate(&self, _ctx: &mut VisitorContext<'_>, meta: MetaInputValue<'_>, pos: Pos, other: T);
}

trait DynValidatorExt {
    fn inner(&self) -> &dyn DynValidate<&Value>;
}

impl DynValidatorExt for DynValidator {
    fn inner(&self) -> &dyn DynValidate<&Value> {
        #[allow(clippy::single_match)]
        match self {
            DynValidator::Length(v) => v,
        }
    }
}

impl DynValidate<&Value> for DynValidator {
    fn validate(&self, ctx: &mut VisitorContext<'_>, meta: MetaInputValue<'_>, pos: Pos, value: &Value) {
        self.inner().validate(ctx, meta, pos, value);
    }
}
