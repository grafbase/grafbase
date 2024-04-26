use {
    super::concrete_type_name_from_parsed_type,
    crate::visitor::{Visitor, VisitorContext},
    engine_parser::{
        types::{OperationDefinition, OperationType},
        Positioned,
    },
    engine_value::Name,
};

#[derive(Default)]
pub struct UploadFile;

impl<'a> Visitor<'a> for UploadFile {
    fn enter_operation_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        _name: Option<&'a Name>,
        operation_definition: &'a Positioned<OperationDefinition>,
    ) {
        for var in &operation_definition.node.variable_definitions {
            let name = concrete_type_name_from_parsed_type(&var.node.var_type.node);
            if let Some(ty) = ctx.registry.lookup_type(name) {
                if operation_definition.node.ty != OperationType::Mutation && ty.name() == "Upload" {
                    ctx.report_error(
                        vec![var.pos],
                        "The Upload type is only allowed to be defined on a mutation",
                    );
                }
            }
        }
    }
}
