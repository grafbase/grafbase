mod query;
mod response;
mod template;

use operation::Variables;
pub(crate) use query::*;
pub(crate) use response::*;
use schema::{ExtensionDirective, InjectionStage, Schema};

use crate::response::ResponseObjectsView;

use super::PartitionFieldArguments;

#[derive(Clone, Copy)]
struct ArgumentsContext<'a> {
    schema: &'a Schema,
    field_arguments: PartitionFieldArguments<'a>,
    variables: &'a Variables,
}

pub(crate) fn create_extension_directive_arguments_view<'ctx, 'resp>(
    schema: &'ctx Schema,
    directive: ExtensionDirective<'ctx>,
    field_arguments: PartitionFieldArguments<'ctx>,
    variables: &'ctx Variables,
    response_objects_view: ResponseObjectsView<'resp>,
) -> (
    ExtensionDirectiveArgumentsQueryView<'ctx>,
    ExtensionDirectiveArgumentsResponseObjectsView<'resp>,
)
where
    'ctx: 'resp,
{
    let ctx = ArgumentsContext {
        schema,
        field_arguments,
        variables,
    };
    let query_view = ExtensionDirectiveArgumentsQueryView { ctx, directive };
    let response_view = ExtensionDirectiveArgumentsResponseObjectsView {
        ctx,
        arguments: directive
            .arguments_with_stage(|stage| matches!(stage, InjectionStage::Response))
            .collect::<Vec<_>>(),
        response_objects_view,
    };

    (query_view, response_view)
}
