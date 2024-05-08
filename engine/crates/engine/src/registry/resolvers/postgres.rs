mod context;
mod request;

use std::{future::Future, pin::Pin};

use async_runtime::make_send_on_wasm;
pub use context::CollectionArgs;
use context::PostgresContext;
use runtime::pg::{PgTransportFactory, PgTransportFactoryError};

use super::{ResolvedValue, ResolverContext};
use crate::{context::ContextExt, ContextField, Error};

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    FindOne,
    FindMany,
    DeleteOne,
    DeleteMany,
    CreateOne,
    CreateMany,
    UpdateOne,
    UpdateMany,
}

impl AsRef<str> for Operation {
    fn as_ref(&self) -> &str {
        match self {
            Self::FindOne => "findOne",
            Self::FindMany => "findMany",
            Self::DeleteOne => "deleteOne",
            Self::DeleteMany => "deleteMany",
            Self::CreateOne => "createOne",
            Self::CreateMany => "createMany",
            Self::UpdateOne => "updateOne",
            Self::UpdateMany => "updateMany",
        }
    }
}

impl From<PgTransportFactoryError> for crate::Error {
    fn from(value: PgTransportFactoryError) -> Self {
        Self::new(value.to_string())
    }
}

pub fn resolve<'a>(
    resolver: &'a registry_v2::resolvers::postgres::PostgresResolver,
    ctx: &'a ContextField<'_>,
    resolver_ctx: &'a ResolverContext<'_>,
) -> Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send + 'a>> {
    Box::pin(make_send_on_wasm(async move {
        let pg_transport_factory = ctx.data::<PgTransportFactory>()?;

        let database_definition = ctx
            .get_postgres_definition(&resolver.directive_name)
            .ok_or(Error::new(format!(
                "pg directive ({}) must exist",
                &resolver.directive_name
            )))?;

        let transport = pg_transport_factory.try_get(&resolver.directive_name).await?;
        let context = PostgresContext::new(ctx, resolver_ctx, database_definition, transport).await?;

        request::execute(context, resolver.operation).await
    }))
}
