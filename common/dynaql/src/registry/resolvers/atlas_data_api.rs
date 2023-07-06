mod error;
mod filter;
mod operation;
mod projection;

pub use operation::OperationType;

use super::{ResolvedValue, ResolverContext};
use crate::{registry::type_kinds::SelectionSetTarget, Context};
use futures_util::Future;
use http::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
use send_wrapper::SendWrapper;
use std::{pin::Pin, sync::Arc};

mod headers {
    pub const API_KEY_HEADER_NAME: &str = "apiKey";
    pub const APPLICATION_JSON_CONTENT_TYPE: &str = "application/json";
}

type JsonMap = serde_json::Map<String, serde_json::Value>;
type Result<T> = std::result::Result<T, error::Error>;

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct AtlasDataApiResolver {
    pub operation_type: OperationType,
    pub app_id: String,
    pub api_key: String,
    pub datasource: String,
    pub database: String,
    pub collection: String,
}

impl AtlasDataApiResolver {
    pub fn resolve(
        &self,
        ctx: &Context<'_>,
        resolver_ctx: &ResolverContext<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedValue>> + Send + '_>> {
        let current_object: SelectionSetTarget<'_> = resolver_ctx.ty.unwrap().try_into().unwrap();
        let selection = ctx.item.node.selection();

        let request_builder = reqwest::Client::new()
            .post(self.url(self.operation_type))
            .header(CONTENT_TYPE, headers::APPLICATION_JSON_CONTENT_TYPE)
            .header(ACCEPT, headers::APPLICATION_JSON_CONTENT_TYPE)
            .header(headers::API_KEY_HEADER_NAME, &self.api_key)
            .header(USER_AGENT, "Grafbase");

        let projection = projection::project(selection, current_object, ctx);

        let mut body = self.body_base();
        body.insert(String::from("projection"), projection.into());

        body.insert(
            String::from("filter"),
            filter::by(current_object, ctx).unwrap(),
        );

        Box::pin(SendWrapper::new(async move {
            let value = request_builder
                .json(&body)
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?
                .take();

            let mut resolved_value = ResolvedValue::new(Arc::new(value));

            if resolved_value.data_resolved.is_null() {
                resolved_value.early_return_null = true;
            }

            Ok(resolved_value)
        }))
    }

    fn url(&self, operation: OperationType) -> url::Url {
        format!(
            "https://data.mongodb-api.com/app/{}/endpoint/data/v1/action/{}",
            self.app_id, operation
        )
        .parse()
        .expect("has to be a real url")
    }

    fn body_base(&self) -> JsonMap {
        let mut map = JsonMap::new();

        map.insert(
            String::from("dataSource"),
            serde_json::Value::String(self.datasource.clone()),
        );

        map.insert(
            String::from("database"),
            serde_json::Value::String(self.database.clone()),
        );

        map.insert(
            String::from("collection"),
            serde_json::Value::String(self.collection.clone()),
        );

        map
    }
}
