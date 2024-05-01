use engine_parser::Pos;
use engine_value::Value;
use meta_type_name::MetaTypeName;

use crate::{dynamic_validators::DynValidate, visitor::Visitor, VisitorContext};

pub struct InputValidationVisitor;

impl<'ctx> Visitor<'ctx, registry_v2::Registry> for InputValidationVisitor {
    fn exit_input_value(
        &mut self,
        ctx: &mut VisitorContext<'ctx, registry_v2::Registry>,
        pos: Pos,
        _expected_type: &Option<MetaTypeName<'_>>,
        value: &Value,
        meta: Option<registry_v2::MetaInputValue<'ctx>>,
    ) {
        if let Some(meta) = meta {
            for validator in meta.validators() {
                validator.validator().validate(ctx, meta, pos, value);
            }
        };
    }
}
