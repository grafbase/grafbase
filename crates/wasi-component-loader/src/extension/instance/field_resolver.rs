use engine_error::GraphqlError;
use futures::future::BoxFuture;
use runtime::extension::Data;

use crate::{extension::api::wit::FieldDefinitionDirective, resources::OwnedOrShared};

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
    #[allow(clippy::type_complexity)]
    fn resolve_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
        inputs: InputList,
    ) -> BoxFuture<'a, wasmtime::Result<Result<Vec<Result<Data, GraphqlError>>, GraphqlError>>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    #[allow(clippy::type_complexity)]
    fn subscription_key<'a>(
        &'a mut self,
        headers: OwnedOrShared<http::HeaderMap>,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<(OwnedOrShared<http::HeaderMap>, Option<Vec<u8>>), GraphqlError>>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn resolve_subscription<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<(), GraphqlError>>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn field_resolver_resolve_next_subscription_item(
        &mut self,
    ) -> BoxFuture<'_, wasmtime::Result<Result<Option<SubscriptionItem>, GraphqlError>>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }
}
