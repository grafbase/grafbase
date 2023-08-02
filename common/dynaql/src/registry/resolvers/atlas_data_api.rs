mod input;
mod normalize;
mod operation;
mod projection;

use std::{pin::Pin, sync::Arc};

use futures_util::Future;
use grafbase_runtime::search::Cursor;
use http::{
    header::{ACCEPT, CONTENT_TYPE, USER_AGENT},
    StatusCode,
};
pub use operation::OperationType;
use send_wrapper::SendWrapper;

use super::{ResolvedPaginationInfo, ResolvedValue, ResolverContext};
use crate::{
    registry::{
        type_kinds::{OutputType, SelectionSetTarget},
        MongoDBConfiguration,
    },
    Context, Error,
};

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
        let config = ctx
            .get_mongodb_config(&self.directive_name)
            .expect("directive must exist");

        let url = format!(
            "{}/app/{}/endpoint/data/v1/action/{}",
            config.host_url, config.app_id, self.operation_type
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
                    let selection_set: SelectionSetTarget<'_> = resolver_ctx.ty.unwrap().try_into().unwrap();

                    let available_fields = selection_set.field_map().unwrap();
                    let selection = ctx.look_ahead().selection_fields();
                    let projection = projection::project(ctx, selection.into_iter(), available_fields)?;

                    body.insert(String::from("projection"), projection);
                    body.insert(String::from("filter"), input::by(ctx)?);
                }
                OperationType::FindMany => {
                    self.find_many(ctx, resolver_ctx, &mut body)?;
                }
                OperationType::InsertOne => {
                    body.insert(String::from("document"), input::input(ctx)?);
                }
                OperationType::DeleteOne => {
                    body.insert(String::from("filter"), input::by(ctx).unwrap());
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

            let pagination = self.pagination(&value);
            let value = self.convert_value(value);

            let mut resolved_value = ResolvedValue::new(Arc::new(value));
            resolved_value.pagination = pagination;

            if resolved_value.data_resolved.is_null() {
                resolved_value.early_return_null = true;
            }

            Ok(resolved_value)
        }))
    }

    fn convert_value(&self, value: serde_json::Value) -> serde_json::Value {
        let mut object = match value {
            serde_json::Value::Object(object) => object,
            value => return value,
        };

        match self.operation_type {
            OperationType::FindMany => object.remove("documents").unwrap_or(serde_json::Value::Null),
            OperationType::FindOne => object.remove("document").unwrap_or(serde_json::Value::Null),
            _ => object.into(),
        }
    }

    fn pagination(&self, value: &serde_json::Value) -> Option<ResolvedPaginationInfo> {
        if OperationType::FindMany != self.operation_type {
            return None;
        }

        let ids = value
            .as_object()
            .and_then(|obj| obj.get("documents"))
            .and_then(serde_json::Value::as_array)
            .and_then(|values| {
                let first = values
                    .first()
                    .and_then(serde_json::Value::as_object)
                    .and_then(|obj| obj.get("_id"))
                    .and_then(serde_json::Value::as_str);

                let last = values
                    .last()
                    .and_then(serde_json::Value::as_object)
                    .and_then(|obj| obj.get("_id"))
                    .and_then(serde_json::Value::as_str);

                first.zip(last)
            });

        let start_cursor = ids.map(|(first, _)| first.as_bytes()).map(Cursor::from);
        let end_cursor = ids.map(|(_, last)| last.as_bytes()).map(Cursor::from);

        Some(ResolvedPaginationInfo {
            start_cursor,
            end_cursor,
            // TODO: implemented in a subsequent PR
            has_next_page: false,
            // TODO: implemented in a subsequent PR
            has_previous_page: false,
        })
    }

    fn find_many(
        &self,
        ctx: &Context<'_>,
        resolver_ctx: &ResolverContext<'_>,
        body: &mut JsonMap,
    ) -> Result<(), Error> {
        let selection_target: SelectionSetTarget<'_> = resolver_ctx.ty.unwrap().try_into().unwrap();

        let selection_type = selection_target
            .field("edges")
            .and_then(|field| ctx.registry().lookup(&field.ty).ok());

        let selection_field = selection_type.as_ref().and_then(|output| output.field("node"));

        let selection_type = selection_field.and_then(|field| ctx.registry().lookup(&field.ty).ok());

        let selection_field_types = selection_type.as_ref().and_then(OutputType::field_map).unwrap();

        let selection = ctx.look_ahead().field("edges").field("node").selection_fields();

        let projection = projection::project(ctx, selection.into_iter(), selection_field_types)?;

        body.insert(String::from("projection"), projection);
        body.insert(String::from("filter"), input::filter(ctx)?);

        if let Some(ordering) = input::order_by(ctx)? {
            body.insert(String::from("sort"), ordering);
        }

        body.insert(String::from("limit"), input::limit(ctx));

        if let Some(skip) = input::skip(ctx) {
            body.insert(String::from("skip"), skip);
        }

        Ok(())
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
        Some(StatusCode::NOT_FOUND) => Error::new(
            "the request was sent to an endpoint that does not exist, please check the connector configuration",
        ),
        Some(StatusCode::UNAUTHORIZED) => {
            Error::new("the request did not include an authorized and enabled Atlas Data API Key")
        }
        Some(StatusCode::INTERNAL_SERVER_ERROR) => {
            Error::new("the Atlas Data API encountered an internal error and could not complete the request")
        }
        _ => Error::new(error.to_string()),
    }
}
