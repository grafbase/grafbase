mod filter;
mod operation;
mod projection;

pub use operation::OperationType;

use super::{ResolvedValue, ResolverContext};
use crate::{registry::type_kinds::SelectionSetTarget, Context, Error};
use futures_util::Future;
use http::{
    header::{ACCEPT, CONTENT_TYPE, USER_AGENT},
    StatusCode,
};
use send_wrapper::SendWrapper;
use std::{pin::Pin, sync::Arc};

mod headers {
    pub const API_KEY_HEADER_NAME: &str = "apiKey";
    pub const APPLICATION_JSON_CONTENT_TYPE: &str = "application/json";
}

type JsonMap = serde_json::Map<String, serde_json::Value>;

/// Resolver for the MongoDB Atlas Data API, which is a MongoDB endpoint using
/// HTTP protocol for transfer.
///
/// # Internal documentation
/// https://www.notion.so/grafbase/MongoDB-Connector-b4d134d2dd0f41ef88dd25cf19143be8
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct AtlasDataApiResolver {
    /// The type of operation to execute in the target.
    pub operation_type: OperationType,
    /// The application id, found in the App Services dashboard.
    pub app_id: String,
    /// The key is generated separately for the Data API.
    pub api_key: String,
    /// The name of the cluster.
    pub datasource: String,
    /// The name of the database in the cluster.
    pub database: String,
    /// The name of the collection in the database.
    pub collection: String,
}

impl AtlasDataApiResolver {
    pub fn resolve(
        &self,
        ctx: &Context<'_>,
        resolver_ctx: &ResolverContext<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send + '_>> {
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

        match self.operation_type {
            OperationType::FindOne => {
                body.insert(String::from("projection"), projection.into());

                body.insert(
                    String::from("filter"),
                    filter::by(current_object, ctx).unwrap(),
                );
            }
        }

        Box::pin(SendWrapper::new(async move {
            let value = request_builder
                .json(&body)
                .send()
                .await
                .map_err(map_err)?
                .error_for_status()
                .map_err(map_err)?
                .json::<serde_json::Value>()
                .await
                .map_err(map_err)?
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

fn map_err(error: reqwest::Error) -> Error {
    match error.status() {
        Some(StatusCode::BAD_REQUEST) => Error::new(format!("the request was malformed: {error}")),
        Some(StatusCode::NOT_FOUND) => {
            Error::new("the request was sent to an endpoint that does not exist, please check the connector configuration")
        }
        Some(StatusCode::UNAUTHORIZED) => {
            Error::new("the request did not include an authorized and enabled Atlas Data API Key")
        }
        Some(StatusCode::INTERNAL_SERVER_ERROR) => Error::new(
            "the Atlas Data API encountered an internal error and could not complete the request",
        ),
        _ => Error::new(error.to_string()),
    }
}
