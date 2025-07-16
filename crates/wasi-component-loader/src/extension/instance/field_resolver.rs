use engine_error::GraphqlError;
use futures::future::BoxFuture;
use runtime::extension::Data;

use crate::{Error, extension::api::wit::FieldDefinitionDirective, resources::Lease};

/// List of inputs to be provided to the extension.
/// The data itself is fully custom and thus will be serialized with serde to cross the Wasm
/// boundary.
#[derive(Default)]
pub(crate) struct InputList(pub(crate) Vec<Vec<u8>>);

impl<S: serde::Serialize> FromIterator<S> for InputList {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|input| crate::cbor::to_vec(&input).unwrap())
                .collect(),
        )
    }
}

pub(crate) type SubscriptionItem = Vec<Result<Data, GraphqlError>>;

#[allow(unused_variables)]
pub(crate) trait FieldResolverExtensionInstance {
    fn resolve_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
        inputs: InputList,
    ) -> BoxFuture<'a, Result<Vec<Result<Data, GraphqlError>>, Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    #[allow(clippy::type_complexity)]
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
    ) -> BoxFuture<'_, Result<Option<SubscriptionItem>, Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }
}
