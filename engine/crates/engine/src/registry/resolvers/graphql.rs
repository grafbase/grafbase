//! Resolve a query to an upstream GraphQL server.
//!
//! The resolver logic is implemented in [`Resolver::resolve`], the options for the resolver are in
//! [`Resolver`].
//!
//! Note that the resolver supports passing headers to the upstream server (e.g. for
//! authentication), but these are fetched from the
//! [`Registry.http_headers`](crate::registry::Registry) field.
//!
//! Defining the resolver within the schema is done through the `@graphql` directive, e.g.:
//!
//! ```text
//! @graphql(
//!   name: "github"
//!   url: "https://api.github.com/graphql"
//!   headers: [{ name: "Authorization", value: "Bearer {{ env.GITHUB_TOKEN }}"}],
//! )
//! ```

mod response;
pub mod serializer;

use std::{
    collections::{BTreeMap, HashMap},
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
};

use async_runtime::make_send_on_wasm;
use dataloader::{DataLoader, Loader, NoCache};
use engine_parser::{
    parse_query,
    types::{
        DocumentOperations, ExecutableDocument, Field, FragmentDefinition, InlineFragment, OperationType, Selection,
        SelectionSet, VariableDefinition,
    },
    Positioned,
};
use engine_value::{ConstValue, Name, Variables};
use futures_util::Future;
use http::{header::USER_AGENT, StatusCode};
use inflector::Inflector;
use internment::ArcIntern;
use tracing::{info_span, Instrument};


use self::serializer::Serializer;
use super::ResolvedValue;
use crate::{
    context::QueryFutureSpawner,
    registry::{
        resolvers::{graphql::response::UpstreamResponse, logged_fetch::send_logged_request},
        type_kinds::SelectionSetTarget,
    },
    QueryPath, QueryPathSegment, ServerError,
};

pub struct QueryBatcher {
    loader: DataLoader<QueryLoader, NoCache>,
}

impl QueryBatcher {
    #[must_use]
    pub fn new() -> Self {
        Self {
            loader: DataLoader::new(QueryLoader),
        }
    }
}

impl Default for QueryBatcher {
    fn default() -> Self {
        Self::new()
    }
}

struct QueryLoader;

#[async_trait::async_trait]
impl Loader<QueryData> for QueryLoader {
    type Value = (UpstreamResponse, StatusCode);
    type Error = Error;

    async fn load(&self, queries: &[QueryData]) -> Result<HashMap<QueryData, Self::Value>, Self::Error> {
        load(queries).await
    }
}

type LoadResult = Result<HashMap<QueryData, (UpstreamResponse, StatusCode)>, Error>;

fn load(queries: &[QueryData]) -> Pin<Box<dyn Future<Output = LoadResult> + Send>> {
    #[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
    struct ResolverDetails {
        name: String,
        url: String,
        headers: Vec<(String, String)>,
        ray_id: String,
        fetch_log_endpoint_url: Option<String>,
    }

    let mut resolver_queries: BTreeMap<_, Vec<QueryData>> = BTreeMap::default();

    for data in queries.iter().cloned() {
        let id = ResolverDetails {
            name: data.resolver_name.clone(),
            url: data.url.clone(),
            headers: data.headers.clone(),
            ray_id: data.ray_id.clone(),
            fetch_log_endpoint_url: data.fetch_log_endpoint_url.clone(),
        };

        resolver_queries.entry(id).or_default().push(data);
    }

    let mut results: HashMap<QueryData, (UpstreamResponse, StatusCode)> = HashMap::default();

    Box::pin(make_send_on_wasm(async move {
        for (resolver, queries) in resolver_queries {
            let (query, aliases) = if queries.len() == 1 {
                (queries[0].query.clone(), vec![HashMap::new()])
            } else {
                group_queries(queries.iter().map(|query| query.query.clone()))
            };

            let mut request_builder = reqwest::Client::new()
                .post(resolver.url.clone())
                .header(USER_AGENT, "Grafbase") // Some APIs (such a GitHub's) require a User-Agent.
                .json(&query)
                .timeout(std::time::Duration::from_secs(30));

            for (name, value) in resolver.headers.clone() {
                request_builder = request_builder.header(name, value);
            }

            let response = send_logged_request(
                &resolver.ray_id,
                resolver.fetch_log_endpoint_url.as_deref(),
                request_builder,
            )
            .await
            .map_err(|e| Error::RequestError(e.to_string()))?;

            let http_status = response.status();

            let upstream_response = UpstreamResponse::from_response_text(
                http_status,
                response
                    .text()
                    .instrument(info_span!("response_data_fetch"))
                    .await
                    .map_err(|e| Error::RequestError(e.to_string())),
            )?;

            for (query, aliases) in queries.into_iter().zip(aliases) {
                if aliases.is_empty() {
                    let UpstreamResponse { data, errors } = upstream_response.clone();
                    let errors = errors
                        .into_iter()
                        .map(|error| translate_error_path(error, &query.local_path, None, query.namespaced))
                        .collect();

                    results.insert(query.clone(), (UpstreamResponse { data, errors }, http_status));
                } else {
                    let errors = filter_and_translate_error_paths_for_group(
                        &aliases,
                        &upstream_response,
                        &query.local_path,
                        query.namespaced,
                    );

                    let data = match &upstream_response.data {
                        serde_json::Value::Object(upstream_data) => {
                            // Take all our aliased fields and un-alias them
                            let mut map = serde_json::Map::new();
                            for (alias, original_name) in &aliases {
                                if let Some(value) = upstream_data.get(alias) {
                                    map.insert(original_name.clone(), value.clone());
                                }
                            }
                            serde_json::Value::Object(map)
                        }
                        _ => serde_json::Value::Null,
                    };

                    results.insert(query.clone(), (UpstreamResponse { data, errors }, http_status));
                }
            }
        }

        Ok(results)
    }))
}

fn filter_and_translate_error_paths_for_group(
    aliases: &HashMap<String, String>,
    upstream_response: &UpstreamResponse,
    local_path: &QueryPath,
    namespaced: bool,
) -> Vec<ServerError> {
    let path_prefixes = aliases
        .iter()
        .map(|(alias, original_name)| {
            (
                vec![QueryPathSegment::Field(ArcIntern::new(alias.clone()))],
                original_name,
            )
        })
        .collect::<Vec<_>>();

    let errors = upstream_response
        .errors
        .iter()
        .filter_map(|error| {
            path_prefixes
                .iter()
                .find_map(|(prefix, original_name)| error.path.starts_with(prefix).then_some(original_name))
                .zip(Some(error))
        })
        .map(|(field_name, error)| translate_error_path(error.clone(), local_path, Some(field_name), namespaced))
        .collect();

    errors
}

fn translate_error_path(
    mut error: ServerError,
    local_path: &QueryPath,
    field_name: Option<&str>,
    namespaced: bool,
) -> ServerError {
    if let Some((field_name, first_segment)) = field_name.zip(error.path.first()) {
        if first_segment != field_name {
            // An alias was probably used, so we need to translate the first key in the remote path
            *error.path.first_mut().unwrap() = QueryPathSegment::Field(ArcIntern::new(field_name.to_string()))
        }
    }

    if namespaced {
        error.path = local_path.iter().cloned().chain(error.path).collect();
    } else {
        // Namespaced connectors are resolved at the field level so we need to drop one more path segment
        error.path = local_path
            .iter()
            .cloned()
            .chain(error.path.into_iter().skip(1))
            .collect();
    }

    // Its more bother than its worth to translate locations imo, so
    // just going to get rid of those
    error.locations = vec![];

    error
}

fn group_queries(queries: impl ExactSizeIterator<Item = Query>) -> (Query, Vec<HashMap<String, String>>) {
    let mut variables = BTreeMap::new();

    let mut all_fragments = HashMap::new();
    let mut all_variable_definitions = HashMap::new();
    let mut all_selections = SelectionSet::default();
    let mut all_directives = HashMap::new();

    let mut root_aliases = Vec::with_capacity(queries.len());

    for Query { query: q, variables: v } in queries {
        let Ok(ExecutableDocument {
            operations: DocumentOperations::Single(operation),
            fragments,
        }) = parse_query(&q)
        else {
            panic!("Expected a valid query with a single operation in it");
        };

        let mut operation = operation.node;

        // Only queries will be grouped.
        debug_assert_eq!(operation.ty, OperationType::Query);

        for def in operation.variable_definitions {
            all_variable_definitions.insert(def.name.clone().into_inner(), def);
        }

        for dir in operation.directives {
            all_directives.insert(dir.name.clone().into_inner(), dir);
        }

        all_fragments.extend(fragments);

        let mut query_root_aliases = HashMap::with_capacity(operation.selection_set.items.len());

        // We need to make sure every field in the root has a unique name in the response
        // Easiest way to do that is just to put a unique alias on _everything_.
        alias_root_fields(
            &mut operation.selection_set.node.items,
            &mut query_root_aliases,
            &all_fragments,
        );

        root_aliases.push(query_root_aliases);
        all_selections.items.append(&mut operation.selection_set.node.items);

        variables.extend(v);
    }

    let mut query = String::new();

    let fragment_definitions = all_fragments.iter().map(|(k, v)| (k, v.as_ref().node)).collect();

    let all_variable_definitions: Vec<_> = all_variable_definitions.into_values().collect();
    let variable_definitions = all_variable_definitions
        .iter()
        .map(|variable_definition| (&variable_definition.node.name.node, &variable_definition.node))
        .collect();

    let mut serializer = Serializer::new(None, fragment_definitions, variable_definitions, &mut query, None);

    let target = Target::SelectionSet(Box::new(all_selections.items.into_iter().map(|v| v.node.clone())));

    serializer.query(target, None).expect("valid grouping of queries");

    (Query { query, variables }, root_aliases)
}

// We use this counter to generate unique aliases for fields to avoid clashes.
// See its use in alias_root_fields below for more details
static ALIAS_COUNTER: AtomicU64 = AtomicU64::new(0);

/// When joining queries together we might have some clashing fields
///
/// In order to make sure the joined query is correct we need to be careful to ensure that every
/// root field is unique.  We do that by sticking aliases on all the root selections, and keeping
/// track of those aliases so we can handle the response.
fn alias_root_fields(
    selections: &mut [Positioned<Selection>],
    alias_map: &mut HashMap<String, String>,
    fragments: &HashMap<Name, Positioned<FragmentDefinition>>,
) {
    for item in selections.iter_mut() {
        match &mut item.node {
            Selection::Field(field) => {
                let original_name = field.alias.clone().unwrap_or_else(|| field.name.clone());
                let alias = format!("f_{}", ALIAS_COUNTER.fetch_add(1, Ordering::Relaxed));

                alias_map.insert(alias.clone(), original_name.to_string());
                field.node.alias = Some(Positioned::new(Name::new(alias), Default::default()));
            }
            Selection::FragmentSpread(spread) => {
                // FragmentSpreads are awwkard - we can't put aliases onto the fragment
                // because it might be used elsewhere.
                //
                // So instead we convert it to an inline fragment instead and do our aliasing on that.

                let Some(fragment) = fragments.get(&spread.node.fragment_name.node) else {
                    // This really shouldn't happen but continue if it does
                    continue;
                };

                let mut inline_fragment = InlineFragment {
                    type_condition: Some(fragment.node.type_condition.clone()),
                    directives: fragment.node.directives.clone(),
                    selection_set: fragment.selection_set.clone(),
                };

                // Alias the fields of that inline fragment, this should also handle any fragment
                // spreads contained within.
                alias_root_fields(&mut inline_fragment.selection_set.node.items, alias_map, fragments);

                // Now replace the fragment spread with the inline fragment.
                item.node = Selection::InlineFragment(Positioned::new(inline_fragment, spread.pos));
            }
            Selection::InlineFragment(inline) => {
                // Recursively alias any fields in this fragment
                alias_root_fields(&mut inline.node.selection_set.node.items, alias_map, fragments);
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct QueryData {
    /// The actual query to be joined with other queries and sent to the remote endpoint.
    query: Query,

    /// A list of headers to add to the remote request.
    headers: Vec<(String, String)>,

    /// The URL of the remote endpoint.
    url: String,

    /// The name of the resolver this query belongs to.
    ///
    /// This data is needed to ensure queries are grouped by resolver. We cannot rely on the URL
    /// alone, as two resolvers might have the same URL, but other properties (such as headers)
    /// might differ.
    resolver_name: String,

    /// Used internally in dev mode.
    ray_id: String,

    /// Used internally in dev mode.
    fetch_log_endpoint_url: Option<String>,

    /// The path prefix that should be used in errors in place of the path we get from remote
    /// servers
    local_path: QueryPath,

    namespaced: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Serialize)]
struct Query {
    query: String,
    variables: BTreeMap<Name, ConstValue>,
}

pub enum Target<'a> {
    SelectionSet(Box<dyn Iterator<Item = Selection> + Send + Sync>),
    Field(Field, registry_v2::MetaField<'a>),
}

/// Resolve the given list of [`Selection`]s at the upstream server, returning the final
/// result.
///
/// # Errors
///
/// See [`Error`] for more details.
#[allow(clippy::too_many_arguments)] // I know clippy, I know
pub(super) fn resolve<'a>(
    resolver: &'a registry_v2::resolvers::graphql::Resolver,
    futures_spawner: QueryFutureSpawner,
    operation: OperationType,
    path: QueryPath,
    ray_id: &'a str,
    fetch_log_endpoint_url: Option<&'a str>,
    headers: &'a [(&'a str, &'a str)],
    fragment_definitions: HashMap<&'a Name, &'a FragmentDefinition>,
    target: Target<'a>,
    current_type: Option<SelectionSetTarget<'a>>,
    mut error_handler: impl FnMut(ServerError) + Send + 'a,
    variables: Variables,
    variable_definitions: HashMap<&'a Name, &'a VariableDefinition>,
    registry: &'a registry_v2::Registry,
    batcher: Option<&'a QueryBatcher>,
) -> Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send + 'a>> {
    let mut query = String::new();

    let prefix = resolver.type_prefix.as_ref().cloned().or(
        // If we don't have a type_prefix we fall back to the namespace.
        // This is mostly for backwards compatability reasons.
        // Every new connector from 2023-10-17 should gave type_prefix set correctly
        resolver.namespace.as_ref().map(inflector::Inflector::to_pascal_case),
    );

    let wrapping_field = match &target {
        Target::SelectionSet(_) => None,
        Target::Field(field, _) if field.alias.is_none() => Some(field.name.node.to_string()),
        Target::Field(field, _) => Some(field.alias.as_ref().unwrap().node.to_string()),
    };

    Box::pin(make_send_on_wasm(async move {
        let mut serializer = Serializer::new(
            prefix.as_deref(),
            fragment_definitions,
            variable_definitions,
            &mut query,
            Some(registry),
        );

        let namespaced = matches!(target, Target::SelectionSet { .. });

        match operation {
            OperationType::Query => serializer.query(target, current_type)?,
            OperationType::Mutation => serializer.mutation(target, current_type)?,
            OperationType::Subscription => return Err(Error::UnsupportedOperation("subscription")),
        };

        let variables = variables
            .into_iter()
            .filter(|(name, _)| serializer.variable_references().any(|reference| reference == name))
            .collect();

        let query_data = QueryData {
            query: Query { query, variables },
            headers: headers
                .iter()
                .copied()
                .map(|(a, b)| (a.to_owned(), b.to_owned()))
                .collect(),
            resolver_name: resolver.name().to_string(),
            url: resolver.url.to_string(),
            ray_id: ray_id.to_owned(),
            fetch_log_endpoint_url: fetch_log_endpoint_url.map(str::to_owned),
            local_path: path,
            namespaced,
        };

        let value = match (batcher, operation) {
            (_, OperationType::Subscription) => return Err(Error::UnsupportedOperation("subscription")),
            (Some(batcher), OperationType::Query) => {
                batcher
                    .loader
                    .load_one(query_data, |f| futures_spawner.spawn(f))
                    .await?
            }
            _ => load(&[query_data]).await?.into_values().next(),
        };

        let Some(value) = value else {
            return Err(Error::MalformedUpstreamResponse);
        };

        let (UpstreamResponse { mut data, errors }, http_status) = value;

        if !http_status.is_success() {
            // If we haven't had a fatal error we should still report the http error
            error_handler(ServerError::new(
                format!("Remote returned http error code: {http_status}"),
                None,
            ));
        }

        errors.into_iter().for_each(error_handler);

        if let Some(prefix) = prefix {
            prefix_result_typename(&mut data, &prefix);
        }

        Ok(ResolvedValue::new(match wrapping_field {
            Some(field) => data
                .as_object_mut()
                .and_then(|m| m.remove(&field))
                .unwrap_or(serde_json::Value::Null),
            None => data,
        }))
    }))
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("the provided operation type is not supported by this resolver: {0}")]
    UnsupportedOperation(&'static str),

    #[error("could not serialize downstream graphql operation: {0}")]
    SerializerError(#[from] serializer::Error),

    #[error("could not deserialize upstream response: {0}")]
    DeserializerError(String),

    #[error("request to upstream server failed: {0}")]
    RequestError(String),

    #[error("couldnt decode JSON from upstream server: {0}")]
    JsonDecodeError(String),

    #[error("received invalid response from upstream server")]
    MalformedUpstreamResponse,

    #[error("received an unexpected status from the downstream server: {0}")]
    HttpErrorResponse(u16, String),
}

impl From<Error> for crate::error::Error {
    fn from(err: Error) -> Self {
        let message = err.to_string();
        if let Error::HttpErrorResponse(_, body) = err {
            crate::error::Error {
                message,
                source: None,
                extensions: Some(crate::ErrorExtensionValues(
                    [("response_content".to_string(), body.into())].into_iter().collect(),
                )),
            }
        } else {
            Self::new(message)
        }
    }
}

/// Before the resolver returns the JSON to the caller, it needs to iterate the JSON, find any
/// `__typename` field, and change the value of that field to contain the prefix defined by the
/// directive that triggered this resolver.
///
/// Without doing so, the caller wouldn't be able to match the typesnames, resulting in invalid
/// data.
fn prefix_result_typename(value: &mut serde_json::Value, prefix: &str) {
    use serde_json::Value::{Array, Object, String};

    match value {
        Array(v) => v.iter_mut().for_each(|v| prefix_result_typename(v, prefix)),
        Object(v) => v.iter_mut().for_each(|(k, v)| match v {
            String(s) if k == "__typename" => *s = format!("{prefix} {s}").to_pascal_case(),
            _ => prefix_result_typename(v, prefix),
        }),
        _ => {}
    }
}
