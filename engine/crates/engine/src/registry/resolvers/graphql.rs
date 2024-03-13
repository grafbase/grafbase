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
    borrow::Cow,
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
use url::Url;

use self::serializer::Serializer;
use super::ResolvedValue;
use crate::{
    registry::{
        resolvers::{graphql::response::UpstreamResponse, logged_fetch::send_logged_request},
        type_kinds::SelectionSetTarget,
        MetaField, Registry,
    },
    ServerError,
};

pub struct QueryBatcher {
    loader: DataLoader<QueryLoader, NoCache>,
}

impl QueryBatcher {
    #[must_use]
    pub fn new() -> Self {
        Self {
            loader: DataLoader::new(QueryLoader, async_runtime::spawn),
        }
    }
}

impl Default for QueryBatcher {
    fn default() -> Self {
        Self::new()
    }
}

// FIXME: Currently the CustomerDeploymentConfig stores MetaField for cache metadata
//        so we must be strictly backward compatible for serialization even though those
//        fields won't be used. Previously we had a `id` field, now we have a `name`.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
#[serde(untagged)]
enum IdOrName {
    LegacyId { id: u16 },
    Name { name: String },
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct Resolver {
    /// A unique name for the given GraphQL resolver instance.
    #[serde(flatten)]
    id_or_name: IdOrName,

    /// The name of this GraphQL resolver instance.
    ///
    /// Each instance is expected to have a unique name, as the name of the instance is used as the
    /// field name within which the root upstream fields are exposed.
    pub namespace: Option<String>,

    /// The prefix for this GraphQL resolver if any.
    ///
    /// If not present this will default to the namespace above, mostly for backwards
    /// compatability reasons.
    ///
    /// This is used by the serializer to make sure there is no collision between global
    /// types. E.g. if a `User` type exists, it won't be overwritten by the same type of the
    /// upstream server, as it'll be prefixed as `MyPrefixUser`.
    pub type_prefix: Option<String>,

    /// The URL of the upstream GraphQL API.
    ///
    /// This should point to the actual query endpoint, not a publicly available playground or any
    /// other destination.
    pub url: Url,
}

impl Resolver {
    #[must_use]
    pub fn new(name: String, url: Url, namespace: Option<String>, type_prefix: Option<String>) -> Self {
        Self {
            id_or_name: IdOrName::Name { name },
            url,
            namespace,
            type_prefix,
        }
    }

    #[must_use]
    pub fn name(&self) -> Cow<'_, String> {
        match &self.id_or_name {
            IdOrName::LegacyId { id } => Cow::Owned(id.to_string()),
            IdOrName::Name { name } => Cow::Borrowed(name),
        }
    }

    #[cfg(test)]
    pub fn stub(name: &str, namespace: impl AsRef<str>, url: impl AsRef<str>) -> Self {
        let namespace = match namespace.as_ref() {
            "" => None,
            v => Some(v.to_owned()),
        };

        Self {
            id_or_name: IdOrName::Name { name: name.to_string() },
            type_prefix: namespace.clone(),
            namespace,
            url: Url::parse(url.as_ref()).expect("valid url"),
        }
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

    let mut resolver_queries: BTreeMap<_, Vec<Query>> = BTreeMap::default();

    for data in queries.iter().cloned() {
        let id = ResolverDetails {
            name: data.resolver_name,
            url: data.url,
            headers: data.headers,
            ray_id: data.ray_id,
            fetch_log_endpoint_url: data.fetch_log_endpoint_url,
        };

        resolver_queries.entry(id).or_default().push(data.query.clone());
    }

    let mut results: HashMap<QueryData, (UpstreamResponse, StatusCode)> = HashMap::default();

    Box::pin(make_send_on_wasm(async move {
        for (resolver, queries) in resolver_queries {
            let (query, aliases) = if queries.len() == 1 {
                (queries[0].clone(), vec![HashMap::new()])
            } else {
                group_queries(queries.clone())
            };

            let mut request_builder = reqwest::Client::new()
                .post(resolver.url.clone())
                .header(USER_AGENT, "Grafbase") // Some APIs (such a GitHub's) require a User-Agent.
                .json(&query);

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
                response.text().await.map_err(|e| Error::RequestError(e.to_string())),
            )?;

            for (query, aliases) in queries.into_iter().zip(aliases) {
                let key = QueryData {
                    query,
                    headers: resolver.headers.clone(),
                    url: resolver.url.clone(),
                    resolver_name: resolver.name.clone(),
                    ray_id: resolver.ray_id.clone(),
                    fetch_log_endpoint_url: resolver.fetch_log_endpoint_url.clone(),
                };

                if aliases.is_empty() {
                    results.insert(key, (upstream_response.clone(), http_status));
                } else {
                    // Take all our aliased fields and un-alias them
                    let data = match &upstream_response.data {
                        serde_json::Value::Object(upstream_data) => {
                            let mut map = serde_json::Map::new();
                            for (alias, original_name) in aliases {
                                if let Some(value) = upstream_data.get(&alias) {
                                    map.insert(original_name, value.clone());
                                }
                            }

                            serde_json::Value::Object(map)
                        }
                        _ => serde_json::Value::Null,
                    };

                    results.insert(
                        key,
                        (
                            UpstreamResponse {
                                data,
                                // Probably need to do something smarter with errors,
                                // but I don't know what.  Just going to duplicate for now.
                                errors: upstream_response.errors.clone(),
                            },
                            http_status,
                        ),
                    );
                }
            }
        }

        Ok(results)
    }))
}

fn group_queries(queries: Vec<Query>) -> (Query, Vec<HashMap<String, String>>) {
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
    let registry = Registry::default();

    let fragment_definitions = all_fragments.iter().map(|(k, v)| (k, v.as_ref().node)).collect();

    let all_variable_definitions: Vec<_> = all_variable_definitions.into_values().collect();
    let variable_definitions = all_variable_definitions
        .iter()
        .map(|variable_definition| (&variable_definition.node.name.node, &variable_definition.node))
        .collect();

    let mut serializer = Serializer::new(None, fragment_definitions, variable_definitions, &mut query, &registry);

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

#[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Serialize)]
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
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Serialize)]
struct Query {
    query: String,
    variables: BTreeMap<Name, ConstValue>,
}

pub enum Target {
    SelectionSet(Box<dyn Iterator<Item = Selection> + Send + Sync>),
    Field(Field, MetaField),
}

impl Resolver {
    /// Resolve the given list of [`Selection`]s at the upstream server, returning the final
    /// result.
    ///
    /// # Errors
    ///
    /// See [`Error`] for more details.
    #[allow(clippy::too_many_arguments)] // I know clippy, I know
    pub(super) fn resolve<'a>(
        &'a self,
        operation: OperationType,
        ray_id: &'a str,
        fetch_log_endpoint_url: Option<&'a str>,
        headers: &'a [(&'a str, &'a str)],
        fragment_definitions: HashMap<&'a Name, &'a FragmentDefinition>,
        target: Target,
        current_type: Option<SelectionSetTarget<'a>>,
        mut error_handler: impl FnMut(ServerError) + Send + 'a,
        variables: Variables,
        variable_definitions: HashMap<&'a Name, &'a VariableDefinition>,
        registry: &'a Registry,
        batcher: Option<&'a QueryBatcher>,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send + 'a>> {
        let mut query = String::new();

        let prefix = self.type_prefix.as_ref().cloned().or(
            // If we don't have a type_prefix we fall back to the namespace.
            // This is mostly for backwards compatability reasons.
            // Every new connector from 2023-10-17 should gave type_prefix set correctly
            self.namespace.as_ref().map(inflector::Inflector::to_pascal_case),
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
                registry,
            );

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
                resolver_name: self.name().to_string(),
                url: self.url.to_string(),
                ray_id: ray_id.to_owned(),
                fetch_log_endpoint_url: fetch_log_endpoint_url.map(str::to_owned),
            };

            let value = match (batcher, operation) {
                (_, OperationType::Subscription) => return Err(Error::UnsupportedOperation("subscription")),
                (Some(batcher), OperationType::Query) => batcher.loader.load_one(query_data).await?,
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
    HttpErrorResponse(u16),
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

#[cfg(test)]
mod tests {
    use engine_parser::parse_query;
    use futures_util::join;
    use indoc::indoc;
    use serde_json::{json, Value};
    use wiremock::{
        matchers::{body_json, header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;
    use crate::registry::builder::RegistryBuilder;

    #[ctor::ctor]
    fn setup_rustls() {
        rustls::crypto::ring::default_provider().install_default().unwrap();
    }

    #[tokio::test]
    async fn resolve() {
        let server = MockServer::start().await;
        let registry = RegistryBuilder::default()
            .build_object("GithubRepository")
            .insert_field("id", "ID!")
            .insert_field("changedFiles", "[String!]!")
            .insert_field("issueOrPullRequest", "GithubIssueOrPr!")
            .insert_field("pullRequest", "GithubPullRequest")
            .finalize_object()
            .build_object("GithubIssue")
            .insert_field("id", "ID!")
            .finalize_object()
            .build_object("GithubPullRequest")
            .insert_field("id", "ID!")
            .insert_field("changedFiles", "[String!]!")
            .finalize_object()
            .insert_union("GithubIssueOrPr", ["GithubIssue", "GithubPullRequest"])
            .build_object("GithubQueries")
            .insert_field("repository", "GithubRepository")
            .finalize_object()
            .build_object("Query")
            .insert_field("github", "GithubQueries")
            .finalize_object()
            .finalize();

        let query = indoc! {r#"
            query {
                github {
                    repository(name: "api", owner: "grafbase") {
                        issueOrPullRequest(number: 2129) {
                            ... on GithubIssue {
                                    id
                            }
                            ... on GithubPullRequest {
                                    id
                                    changedFiles
                            }
                        }
                    }
                }
            }"#};

        let response = json!({
            "data": {
                "github": {
                    "repository": {
                        "issueOrPullRequest": {
                            "id": "PR_kwDOEn_gEs5PlTvR",
                            "changedFiles": 1
                        }
                    }
                }
            }
        });

        Mock::given(method("POST"))
            .and(path("/"))
            .and(header("User-Agent", "Grafbase"))
            .and(header("Authorization", "Bearer FOOBAR"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response.clone()))
            .expect(1)
            .mount(&server)
            .await;

        let result = resolve_registry(
            Resolver::stub("Test", "myApi", server.uri()),
            registry.clone(),
            None,
            query,
        )
        .await;

        assert_eq!(result.as_ref().err(), None);

        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!(result.unwrap());
        });
    }

    struct FnMatcher<T: Fn(&wiremock::Request) -> bool + Send + Sync>(T);

    impl<T: Fn(&wiremock::Request) -> bool + Send + Sync> wiremock::Match for FnMatcher<T> {
        fn matches(&self, request: &wiremock::Request) -> bool {
            self.0(request)
        }
    }

    fn request_fn<T: Fn(&wiremock::Request) -> bool + Send + Sync>(func: T) -> FnMatcher<T> {
        FnMatcher(func)
    }

    #[tokio::test]
    async fn batching_queries() {
        let server = MockServer::start().await;
        let batcher = QueryBatcher::new();

        // 1. Stub our `Registry` type.
        let registry = RegistryBuilder::default()
            .insert_object("FooObject")
            .insert_object("BarObject")
            .build_object("Query")
            .insert_field("foo", "FooObject")
            .insert_field("bar", "BarObject")
            .finalize_object()
            .finalize();

        // 2. We have two query fields, but expect a single API call due to batching.
        Mock::given(method("POST"))
            .and(request_fn(|req| {
                let body = req.body_json::<Value>().unwrap().to_string();
                body.contains("foo\\n\\tbar") || body.contains("bar\\n\\tfoo")
            }))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {} })))
            .expect(1)
            .mount(&server)
            .await;

        // 3. Perform two different queries in parallel, using the same `QueryBatcher`.
        let foo = resolve_registry(
            Resolver::stub("Test", "", server.uri()),
            registry.clone(),
            Some(&batcher),
            "query { foo }",
        );

        let bar = resolve_registry(
            Resolver::stub("Test", "", server.uri()),
            registry,
            Some(&batcher),
            "query { bar }",
        );

        let _results = join!(foo, bar);

        // 4. Validate mocking expectations.
        server.verify().await;
    }

    #[tokio::test]
    async fn batching_named_queries() {
        let server = MockServer::start().await;
        let batcher = QueryBatcher::new();

        // 1. Stub our `Registry` type.
        let registry = RegistryBuilder::default()
            .insert_object("FooObject")
            .insert_object("BarObject")
            .build_object("Query")
            .insert_field("foo", "FooObject")
            .insert_field("bar", "BarObject")
            .finalize_object()
            .finalize();

        // 2. We have two query fields, but expect a single API call due to batching.
        Mock::given(method("POST"))
            .and(request_fn(|req| {
                let body = req.body_json::<Value>().unwrap().to_string();
                body.contains("foo\\n\\tbar") || body.contains("bar\\n\\tfoo")
            }))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {} })))
            .expect(1)
            .mount(&server)
            .await;

        // 3. Perform two different queries in parallel, using the same `QueryBatcher`.
        let foo = resolve_registry(
            Resolver::stub("Test", "", server.uri()),
            registry.clone(),
            Some(&batcher),
            "query Hello { foo }",
        );

        let bar = resolve_registry(
            Resolver::stub("Test", "", server.uri()),
            registry,
            Some(&batcher),
            "query World { bar }",
        );

        let _results = join!(foo, bar);

        // 4. Validate mocking expectations.
        server.verify().await;
    }

    #[tokio::test]
    async fn mutations_are_never_batched() {
        let server = MockServer::start().await;
        let batcher = QueryBatcher::new();

        // 1. Stub our `Registry` type.
        let registry = RegistryBuilder::default()
            .insert_object("FooObject")
            .insert_object("BarObject")
            .build_object("Mutation")
            .insert_field("foo", "FooObject")
            .insert_field("bar", "BarObject")
            .finalize_object()
            .finalize();

        // 2. Mutations are processed sequentially, resulting in two invidual requests.
        Mock::given(method("POST"))
            .and(body_json(json!({"query":"mutation {\n\tfoo\n}\n","variables":{}})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {} })))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(body_json(json!({"query":"mutation {\n\tbar\n}\n","variables":{}})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {} })))
            .expect(1)
            .mount(&server)
            .await;

        // 3. Perform two different queries in parallel, using the same `QueryBatcher`.
        let foo = resolve_registry(
            Resolver::stub("Test", "", server.uri()),
            registry.clone(),
            Some(&batcher),
            "mutation { foo }",
        );

        let bar = resolve_registry(
            Resolver::stub("Test", "", server.uri()),
            registry,
            Some(&batcher),
            "mutation { bar }",
        );

        let (a, b) = join!(foo, bar);
        assert_eq!(a.err(), None);
        assert_eq!(b.err(), None);

        // 4. Validate mocking expectations.
        server.verify().await;
    }

    #[tokio::test]
    async fn batching_queries_with_variables() {
        let server = MockServer::start().await;
        let batcher = QueryBatcher::new();

        // 1. Stub our `Registry` type.
        let registry = RegistryBuilder::default()
            .insert_object("FooObject")
            .insert_object("BarObject")
            .build_object("Query")
            .build_field("foo", "FooObject")
            .insert_argument("id", "ID!")
            .finalize_field()
            .build_field("bar", "BarObject")
            .insert_argument("id", "ID!")
            .finalize_field()
            .finalize_object()
            .finalize();

        // 2. We have two query fields, but expect a single API call due to batching.
        Mock::given(method("POST"))
            .and(request_fn(|req| {
                let body = req.body_json::<Value>().unwrap().to_string();

                let vars = body.contains("query($foo: ID, $bar: ID)") || body.contains("query($bar: ID, $foo: ID)");

                let fields = body.contains("foo(id: $foo)\\n\\tbar(id: $bar)")
                    || body.contains("bar(id: $bar)\\n\\tfoo(id: $foo)");

                vars && fields
            }))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {} })))
            .expect(1)
            .mount(&server)
            .await;

        // 3. Perform two different queries in parallel, using the same `QueryBatcher`.
        let foo = resolve_registry(
            Resolver::stub("Test", "", server.uri()),
            registry.clone(),
            Some(&batcher),
            "query Foo($foo: ID) { foo(id: $foo) }",
        );

        let bar = resolve_registry(
            Resolver::stub("Test", "", server.uri()),
            registry,
            Some(&batcher),
            "query Foo($bar: ID) { bar(id: $bar) }",
        );

        let (a, b) = join!(foo, bar);
        assert_eq!(a.err(), None);
        assert_eq!(b.err(), None);

        // 4. Validate mocking expectations.
        server.verify().await;
    }

    async fn resolve_registry(
        resolver: Resolver,
        registry: Registry,
        batcher: Option<&QueryBatcher>,
        query: impl AsRef<str>,
    ) -> Result<Value, Error> {
        let mut errors = vec![];
        let headers = vec![("Authorization", "Bearer FOOBAR")];
        let document = parse_query(query).unwrap();

        let fragment_definitions = document.fragments.iter().map(|(k, v)| (k, v.as_ref().node)).collect();

        let operation = document
            .operations
            .iter()
            .next()
            .expect("at least one operation")
            .1
            .clone()
            .into_inner();

        let variable_definitions = operation
            .variable_definitions
            .iter()
            .map(|variable_definition| (&variable_definition.node.name.node, &variable_definition.node))
            .collect();

        let operation_type = operation.ty;

        let current_type = match operation.ty {
            OperationType::Query => registry.lookup_by_str("Query").unwrap().try_into().unwrap(),
            OperationType::Mutation => registry.lookup_by_str("Mutation").unwrap().try_into().unwrap(),
            OperationType::Subscription => unimplemented!(),
        };

        let target = Target::SelectionSet(Box::new(
            operation.selection_set.node.items.clone().into_iter().map(|v| v.node),
        ));

        let error_handler = |error| errors.push(error);

        let data = resolver
            .resolve(
                operation_type,
                "",
                None,
                &headers,
                fragment_definitions,
                target,
                Some(current_type),
                error_handler,
                Variables::default(),
                variable_definitions,
                &registry,
                batcher,
            )
            .await?
            .data_resolved()
            .clone();

        let response = if errors.is_empty() {
            json!({ "data": data })
        } else {
            json!({ "data": data, "errors": errors.clone() })
        };

        Ok(response)
    }

    #[test]
    fn backward_compatibility_serde() {
        assert_eq!(
            serde_json::from_str::<Resolver>(
                r#"
                {
                    "id": 1,
                    "url": "https://example.com",
                    "namespace": "prefix"
                }
                "#
            )
            .unwrap(),
            Resolver {
                id_or_name: IdOrName::LegacyId { id: 1 },
                url: "https://example.com".parse().unwrap(),
                namespace: Some("prefix".into()),
                type_prefix: None
            }
        );

        assert_eq!(
            serde_json::from_str::<Resolver>(
                r#"
                {
                    "name": "hello",
                    "url": "https://example.com",
                    "namespace": "prefix"
                }
                "#
            )
            .unwrap(),
            Resolver {
                id_or_name: IdOrName::Name {
                    name: "hello".to_string()
                },
                url: "https://example.com".parse().unwrap(),
                namespace: Some("prefix".into()),
                type_prefix: None
            }
        );
    }
}
