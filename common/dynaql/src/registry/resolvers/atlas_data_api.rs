mod filter;
mod operation;
mod projection;

pub use operation::OperationType;

use super::{ResolvedValue, ResolverContext};
use crate::{
    registry::{type_kinds::SelectionSetTarget, MongoDBConfiguration},
    Context, Error,
};
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
    pub const APPLICATION_EJSON_CONTENT_TYPE: &str = "application/ejson";
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
    pub directive_name: String,
    pub collection: String,
}

impl AtlasDataApiResolver {
    pub fn resolve<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        resolver_ctx: &'a ResolverContext<'_>,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send + 'a>> {
        let selection_set: SelectionSetTarget<'_> = resolver_ctx.ty.unwrap().try_into().unwrap();
        let selection = ctx.item.node.selection();

        let config = ctx
            .get_mongodb_config(&self.directive_name)
            .expect("directive must exist");

        let url = format!(
            "https://data.mongodb-api.com/app/{}/endpoint/data/v1/action/{}",
            config.app_id, self.operation_type
        );

        let request_builder = reqwest::Client::new()
            .post(url)
            .header(CONTENT_TYPE, headers::APPLICATION_EJSON_CONTENT_TYPE)
            .header(ACCEPT, headers::APPLICATION_JSON_CONTENT_TYPE)
            .header(headers::API_KEY_HEADER_NAME, &config.api_key)
            .header(USER_AGENT, "Grafbase");

        Box::pin(SendWrapper::new(async move {
            let mut body = self.base_body(config);

            match self.operation_type {
                OperationType::FindOne => {
                    let projection = projection::project(selection, selection_set, ctx)?;
                    body.insert(String::from("projection"), projection.into());
                    body.insert(String::from("filter"), filter::by(ctx)?);
                }
                OperationType::InsertOne => {
                    body.insert(String::from("document"), filter::input(ctx)?);
                }
            }

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

    fn base_body(&self, config: &MongoDBConfiguration) -> JsonMap {
        let mut body = JsonMap::new();

        body.insert(
            String::from("dataSource"),
            serde_json::Value::String(config.data_source.clone()),
        );

        body.insert(
            String::from("database"),
            serde_json::Value::String(config.database.clone()),
        );

        body.insert(
            String::from("collection"),
            serde_json::Value::String(self.collection.clone()),
        );

        body
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
