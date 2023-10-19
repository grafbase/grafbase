mod context;
mod request;

use std::{future::Future, pin::Pin};

use async_runtime::make_send_on_wasm;
pub use context::CollectionArgs;
use context::PostgresContext;
use runtime::pg::PgTransportFactory;

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

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct PostgresResolver {
    pub(super) operation: Operation,
    pub(super) directive_name: String,
}

impl PostgresResolver {
    pub fn new(operation: Operation, directive_name: &str) -> Self {
        Self {
            operation,
            directive_name: directive_name.to_string(),
        }
    }

    pub fn resolve<'a>(
        &'a self,
        ctx: &'a ContextField<'_>,
        resolver_ctx: &'a ResolverContext<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send + 'a>> {
        Box::pin(make_send_on_wasm(async move {
            let pg_transport_factory = ctx.data::<PgTransportFactory>()?;

            let database_definition = ctx
                .get_postgres_definition(&self.directive_name)
                .ok_or(Error::new(format!(
                    "pg directive ({}) must exist",
                    &self.directive_name
                )))?;

            let transport = pg_transport_factory
                .try_new(&self.directive_name, database_definition)
                .await?;

            let context = PostgresContext::new(ctx, resolver_ctx, database_definition, transport).await?;
            request::execute(context, self.operation).await
        }))
    }
}
