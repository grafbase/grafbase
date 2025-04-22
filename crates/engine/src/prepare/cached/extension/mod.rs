mod query;
mod response;
mod template;

use operation::Variables;
pub(crate) use query::*;
pub(crate) use response::*;
use schema::{ExtensionDirective, ExtensionDirectiveArgumentsStaticView, InjectionStage, Schema};

use crate::response::ResponseObjectsView;

use super::PlanFieldArguments;

#[derive(Clone, Copy)]
pub(crate) struct ArgumentsContext<'a> {
    schema: &'a Schema,
    field_arguments: PlanFieldArguments<'a>,
    variables: &'a Variables,
}

pub(crate) fn create_extension_directive_query_view<'ctx>(
    schema: &'ctx Schema,
    directive: ExtensionDirective<'ctx>,
    field_arguments: PlanFieldArguments<'ctx>,
    variables: &'ctx Variables,
) -> ExtensionDirectiveArgumentsQueryView<'ctx> {
    let ctx = ArgumentsContext {
        schema,
        field_arguments,
        variables,
    };

    ExtensionDirectiveArgumentsQueryView { ctx, directive }
}

pub(crate) fn create_extension_directive_response_view<'ctx, 'resp>(
    schema: &'ctx Schema,
    directive: ExtensionDirective<'ctx>,
    field_arguments: PlanFieldArguments<'ctx>,
    variables: &'ctx Variables,
    response_objects_view: ResponseObjectsView<'resp>,
) -> ExtensionDirectiveArgumentsResponseObjectsView<'resp>
where
    'ctx: 'resp,
{
    let ctx = ArgumentsContext {
        schema,
        field_arguments,
        variables,
    };

    let arguments = directive
        .arguments_with_stage(|stage| matches!(stage, InjectionStage::Response))
        .collect::<Vec<_>>();

    ExtensionDirectiveArgumentsResponseObjectsView {
        ctx,
        arguments,
        response_objects_view,
    }
}

pub(crate) enum QueryOrStaticExtensionDirectiveArugmentsView<'a> {
    Query(ExtensionDirectiveArgumentsQueryView<'a>),
    Static(ExtensionDirectiveArgumentsStaticView<'a>),
}

impl serde::Serialize for QueryOrStaticExtensionDirectiveArugmentsView<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            QueryOrStaticExtensionDirectiveArugmentsView::Query(view) => view.serialize(serializer),
            QueryOrStaticExtensionDirectiveArugmentsView::Static(view) => view.serialize(serializer),
        }
    }
}
