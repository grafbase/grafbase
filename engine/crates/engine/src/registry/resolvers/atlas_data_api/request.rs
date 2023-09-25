mod query;

use super::OperationType;
use crate::{
    registry::{
        resolvers::{ResolvedValue, ResolverContext},
        MongoDBConfiguration,
    },
    ContextExt, ContextField, Error,
};
use http::{
    header::{ACCEPT, CONTENT_TYPE, USER_AGENT},
    StatusCode,
};
use query::{AtlasQuery, DeleteMany, DeleteOne, FindMany, FindOne, InsertMany, InsertOne, UpdateMany, UpdateOne};
use serde::Serialize;
use serde_json::Value;

mod headers {
    pub const API_KEY_HEADER_NAME: &str = "apiKey";
    pub const APPLICATION_JSON_CONTENT_TYPE: &str = "application/json";
    pub const APPLICATION_EJSON_CONTENT_TYPE: &str = "application/ejson";
}

pub(super) async fn execute(
    ctx: &ContextField<'_>,
    resolver_ctx: &ResolverContext<'_>,
    config: &MongoDBConfiguration,
    collection: &str,
    operation_type: OperationType,
) -> Result<ResolvedValue, Error> {
    let query: AtlasQuery = match operation_type {
        OperationType::FindOne => FindOne::new(ctx, resolver_ctx)?.into(),
        OperationType::FindMany => FindMany::new(ctx, resolver_ctx)?.into(),
        OperationType::InsertOne => InsertOne::new(ctx)?.into(),
        OperationType::DeleteOne => DeleteOne::new(ctx)?.into(),
        OperationType::DeleteMany => DeleteMany::new(ctx)?.into(),
        OperationType::InsertMany => InsertMany::new(ctx)?.into(),
        OperationType::UpdateOne => UpdateOne::new(ctx)?.into(),
        OperationType::UpdateMany => UpdateMany::new(ctx)?.into(),
    };

    // In some cases, if our input is empty, we want to short-circuit here and
    // return an early response.
    //
    // In cases of update statements, the user might send us `unset: false` for a
    // field, which we will take out from the final query. If the `update` statement
    // is then empty, it will _remove all fields_ from the document(s).
    if query.is_empty() {
        return Ok(ResolvedValue::new(query.empty_response()));
    }

    let request = AtlasRequest {
        data_source: &config.data_source,
        database: &config.database,
        collection,
        query,
    };

    let url = format!("{}/action/{}", config.url, operation_type);
    let graphql_request_execution_context = ctx.data::<runtime::GraphqlRequestExecutionContext>()?;
    let ray_id = &graphql_request_execution_context.ray_id;
    let fetch_log_endpoint_url = graphql_request_execution_context.fetch_log_endpoint_url.as_deref();

    let request_builder = reqwest::Client::new()
        .post(url)
        .header(CONTENT_TYPE, headers::APPLICATION_EJSON_CONTENT_TYPE)
        .header(ACCEPT, headers::APPLICATION_JSON_CONTENT_TYPE)
        .header(headers::API_KEY_HEADER_NAME, &config.api_key)
        .header(USER_AGENT, "Grafbase")
        .json(&request);

    let value = super::super::logged_fetch::send_logged_request(ray_id, fetch_log_endpoint_url, request_builder)
        .await
        .map_err(map_err)?
        .error_for_status()
        .map_err(map_err)?
        .json::<serde_json::Value>()
        .await
        .map_err(map_err)?
        .take();

    request.convert_result(ctx, resolver_ctx, value)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AtlasRequest<'a> {
    data_source: &'a str,
    database: &'a str,
    collection: &'a str,
    #[serde(flatten)]
    query: AtlasQuery,
}

impl<'a> AtlasRequest<'a> {
    fn convert_result(
        &self,
        ctx: &ContextField<'_>,
        resolver_ctx: &ResolverContext<'_>,
        mut value: Value,
    ) -> Result<ResolvedValue, Error> {
        let result = match self.query {
            AtlasQuery::FindOne(ref query) => query.convert_result(&mut value),
            AtlasQuery::FindMany(ref query) => query.convert_result(ctx, resolver_ctx, &mut value)?,
            _ => ResolvedValue::new(value),
        };

        Ok(result)
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
