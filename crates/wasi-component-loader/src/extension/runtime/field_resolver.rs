use crate::{
    cbor,
    extension::{InputList, api::wit},
    resources::OwnedOrShared,
    wasmsafe,
};

use super::{
    EngineWasmExtensions,
    subscription::{DeduplicatedSubscription, UniqueSubscription},
};

use engine_error::GraphqlError;
use engine_schema::{ExtensionDirective, FieldDefinition};
use futures::stream::BoxStream;
use runtime::extension::{Anything, Data, FieldResolverExtension};
use std::future::Future;

impl FieldResolverExtension for EngineWasmExtensions {
    #[allow(clippy::manual_async_fn)]
    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        subgraph_headers: http::HeaderMap,
        directive_arguments: impl Anything<'ctx>,
        inputs: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        let inputs = InputList::from_iter(inputs);

        async move {
            let mut instance = self.get(directive.extension_id).await?;
            let subgraph = directive.subgraph().expect("Must be present for resolvers");

            let directive = wit::FieldDefinitionDirective {
                name: directive.name(),
                site: wit::FieldDefinitionDirectiveSite {
                    parent_type_name: field_definition.parent_entity().name(),
                    field_name: field_definition.name(),
                },
                arguments: &cbor::to_vec(directive_arguments).unwrap(),
            };

            wasmsafe!(
                instance
                    .resolve_field(subgraph_headers, subgraph.name(), directive, inputs)
                    .await
            )
        }
    }

    async fn resolve_subscription_field<'ctx, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        subgraph_headers: http::HeaderMap,
        directive_arguments: impl Anything<'ctx>,
    ) -> Result<BoxStream<'f, Result<Data, GraphqlError>>, GraphqlError>
    where
        'ctx: 'f,
    {
        let mut instance = self.get(directive.extension_id).await?;
        let subgraph = directive.subgraph().expect("Must be present for resolvers");
        let arguments = &cbor::to_vec(directive_arguments).unwrap();

        let site = wit::FieldDefinitionDirectiveSite {
            parent_type_name: field_definition.parent_entity().name(),
            field_name: field_definition.name(),
        };

        let directive = wit::FieldDefinitionDirective {
            name: directive.name(),
            site,
            arguments,
        };

        instance.recyclable = false;
        let (headers, key) = wasmsafe!(
            instance
                .subscription_key(
                    OwnedOrShared::Owned(subgraph_headers),
                    subgraph.name(),
                    directive.clone()
                )
                .await
        )?;

        let headers = headers.into_inner().unwrap();

        match key {
            Some(key) => {
                let subscription = DeduplicatedSubscription {
                    extensions: self.clone(),
                    instance,
                    headers,
                    key,
                    subgraph,
                    directive,
                };

                subscription.resolve().await
            }
            None => {
                let subscription = UniqueSubscription {
                    instance,
                    headers,
                    subgraph,
                    directive,
                };

                subscription.resolve().await
            }
        }
    }
}
