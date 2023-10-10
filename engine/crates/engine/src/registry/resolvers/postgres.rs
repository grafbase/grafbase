mod context;
mod request;

use async_runtime::make_send_on_wasm;
pub use context::CollectionArgs;

use super::{ResolvedValue, ResolverContext};
use crate::{ContextField, Error};
use context::PostgresContext;
use std::{future::Future, pin::Pin};

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
            let context = PostgresContext::new(ctx, resolver_ctx, &self.directive_name)?;
            request::execute(context, self.operation).await
        }))
    }
}
