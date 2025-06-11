use engine_error::GraphqlError;
use futures::future::BoxFuture;

use crate::{
    Error,
    extension::api::wit::{ArgumentsId, Directive, Field, FieldId, Response, SubscriptionItem},
};

#[allow(unused_variables)]
pub(crate) trait ResolverExtensionInstance {
    fn prepare<'a>(
        &'a mut self,
        subgraph_name: &'a str,
        directive: Directive<'a>,
        field_id: FieldId,
        fields: &'a [Field<'a>],
    ) -> BoxFuture<'a, Result<Result<Vec<u8>, GraphqlError>, Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn resolve<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        prepared: &'a [u8],
        arguments: &'a [(ArgumentsId, &'a [u8])],
    ) -> BoxFuture<'a, Result<Response, Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    #[allow(clippy::type_complexity)]
    fn create_subscription<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        prepared: &'a [u8],
        arguments: &'a [(ArgumentsId, &'a [u8])],
    ) -> BoxFuture<'a, Result<Result<Option<Vec<u8>>, GraphqlError>, Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn drop_subscription<'a>(&'a mut self) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }

    fn resolve_next_subscription_item(
        &mut self,
    ) -> BoxFuture<'_, Result<Result<Option<SubscriptionItem>, GraphqlError>, Error>> {
        Box::pin(async { unreachable!("Not supported by this SDK") })
    }
}
