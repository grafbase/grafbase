mod field_resolver;
mod subscription_resolver;

use field_resolver::FieldResolverExtensionRequest;
use futures_lite::FutureExt;
use runtime::extension::{ExtensionFieldDirective, ExtensionRuntime};
use schema::{ExtensionDirectiveId, FieldResolverExtensionDefinition};
use subscription_resolver::SubscriptionResolverExtensionRequest;
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{Plan, create_extension_directive_arguments_view, create_extension_directive_response_view},
    response::{ResponseObjectsView, SubgraphResponse},
};

use super::Resolver;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct FieldResolverExtension {
    pub directive_id: ExtensionDirectiveId,
}

impl FieldResolverExtension {
    pub(in crate::resolver) fn prepare(definition: FieldResolverExtensionDefinition<'_>) -> Resolver {
        Resolver::FieldResolverExtension(Self {
            directive_id: definition.directive_id,
        })
    }

    pub(in crate::resolver) fn prepare_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
    ) -> SubscriptionResolverExtensionRequest<'ctx> {
        let directive = self.directive_id.walk(ctx.schema());

        let field = plan
            .selection_set()
            .fields()
            .next()
            .expect("At least one field must be present");

        let field_definition = field.definition();

        let query_view =
            create_extension_directive_arguments_view(ctx.schema(), directive, field.arguments(), ctx.variables());

        let extension_directive = ExtensionFieldDirective {
            extension_id: directive.extension_id,
            subgraph: directive.subgraph(),
            field: field_definition,
            name: directive.name(),
            arguments: query_view,
        };

        let future = ctx
            .engine
            .runtime
            .extensions()
            .resolve_field_subscription(ctx.hooks_context, extension_directive)
            .boxed();

        SubscriptionResolverExtensionRequest { field, future }
    }

    pub(in crate::resolver) fn prepare_request<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        root_response_objects: ResponseObjectsView<'_>,
        subgraph_response: SubgraphResponse,
    ) -> FieldResolverExtensionRequest<'ctx> {
        let directive = self.directive_id.walk(ctx.schema());
        let field = plan
            .selection_set()
            .fields()
            .next()
            .expect("At least one field must be present");

        let field_definition = field.definition();

        let query_view =
            create_extension_directive_arguments_view(ctx.schema(), directive, field.arguments(), ctx.variables());

        let response_view =
            create_extension_directive_response_view(query_view.ctx, directive, root_response_objects.clone());

        let extension_directive = ExtensionFieldDirective {
            extension_id: directive.extension_id,
            subgraph: directive.subgraph(),
            field: field_definition,
            name: directive.name(),
            arguments: query_view,
        };

        let future = ctx
            .engine
            .runtime
            .extensions()
            .resolve_field(ctx.hooks_context, extension_directive, response_view.iter())
            .boxed();

        let input_object_refs = root_response_objects.into_input_object_refs();

        FieldResolverExtensionRequest {
            field,
            subgraph_response,
            input_object_refs,
            future,
        }
    }
}
