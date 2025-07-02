use engine_error::GraphqlError;
use futures::future::BoxFuture;
use runtime::extension::Data;

use crate::{
    Error,
    extension::{FieldResolverExtensionInstance, InputList, api::wit::FieldDefinitionDirective},
    resources::Lease,
};

#[allow(unused_variables)]
impl FieldResolverExtensionInstance for super::ExtensionInstanceSince0_19_0 {
    fn resolve_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
        inputs: InputList,
    ) -> BoxFuture<'a, Result<Vec<Result<Data, GraphqlError>>, Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn subscription_key<'a>(
        &'a mut self,
        headers: Lease<http::HeaderMap>,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, Result<(Lease<http::HeaderMap>, Option<Vec<u8>>), Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn resolve_subscription<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn field_resolver_resolve_next_subscription_item(
        &mut self,
    ) -> BoxFuture<'_, Result<Option<Vec<Result<Data, GraphqlError>>>, Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }
}
