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

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(warnings)]
#![deny(let_underscore)]
#![deny(nonstandard_style)]
#![deny(unused)]
#![deny(rustdoc::all)]

pub mod serializer;

use std::{collections::HashMap, sync::Arc};

use dynaql_parser::types::{FragmentDefinition, OperationType, Selection};
use dynaql_value::Name;
use http::header::USER_AGENT;
use inflector::Inflector;
use url::Url;

use crate::ServerError;

use self::serializer::Serializer;

use super::ResolvedValue;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct Resolver {
    /// The name of this GraphQL resolver instance.
    ///
    /// Each instance is expected to have a unique name, as the name of the instance is used as the
    /// field name within which the root upstream fields are exposed.
    ///
    /// Additionally, it is use by the serializer to make sure there is no collision between global
    /// types. E.g. if a `User` type exists, it won't be overwritten by the same type of the
    /// upstream server, as it'll be prefixed as `MyPrefixUser`.
    ///
    /// Note that this *only* affects global types. Anything that's scoped at a lower level is kept
    /// as-is.
    pub api_name: String,

    /// The URL of the upstream GraphQL API.
    ///
    /// This should point to the actual query endpoint, not a publicly available playground or any
    /// other destination.
    pub url: Url,
}

#[derive(serde::Serialize)]
struct Query {
    query: String,
    variables: HashMap<String, String>,
}

impl Resolver {
    /// Resolve the given list of [`Selection`]s at the upstream server, returning the final
    /// result.
    ///
    /// # Errors
    ///
    /// See [`Error`] for more details.
    pub async fn resolve(
        &self,
        operation: OperationType,
        headers: &[(String, String)],
        fragment_definitions: HashMap<&Name, &FragmentDefinition>,
        selection_set: impl Iterator<Item = &Selection>,
        error_handler: impl FnMut(ServerError),
    ) -> Result<ResolvedValue, Error> {
        let mut request_builder = reqwest::Client::new()
            .post(self.url.clone())
            .header(USER_AGENT, "Grafbase"); /* Some APIs (such a GitHub's) require a User-Agent
                                             header */

        for (name, value) in headers {
            request_builder = request_builder.header(name, value);
        }

        let mut query = String::new();
        let prefix = self.api_name.to_pascal_case();

        {
            let mut serializer = Serializer::new(&prefix, fragment_definitions, &mut query);
            match operation {
                OperationType::Query => serializer.query(selection_set)?,
                OperationType::Mutation => serializer.mutation(selection_set)?,
                OperationType::Subscription => {
                    return Err(Error::UnsupportedOperation("subscription"))
                }
            };
        }

        let mut result = request_builder
            .json(&Query {
                query,
                variables: HashMap::new(),
            })
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?
            .take();

        // Merge any upstream GraphQL errors.
        if let Some(errors) = result.get_mut("errors") {
            serde_json::from_value(errors.take())
                .map_err(|_| Error::MalformedUpstreamResponse)
                .map(|errors: Vec<ServerError>| {
                    errors.into_iter().for_each(error_handler);
                })?;
        }

        let mut data = result
            .get_mut("data")
            .ok_or(Error::MalformedUpstreamResponse)?
            .take();

        prefix_result_typename(&mut data, &prefix);

        Ok(ResolvedValue::new(Arc::new(data)))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("the provided operation type is not supported by this resolver: {0}")]
    UnsupportedOperation(&'static str),

    #[error("could not serialize execution plan: {0}")]
    SerializerError(#[from] serializer::Error),

    #[error("request to upstream server failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("received invalid response from upstream server")]
    MalformedUpstreamResponse,
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
    use indoc::indoc;
    use serde_json::{json, Value};
    use wiremock::MockServer;

    use super::*;

    #[tokio::test]
    async fn resolve() {
        let server = MockServer::start().await;

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

        let result = run_resolve(&server, query, response).await.unwrap();

        insta::assert_json_snapshot!(result)
    }

    async fn run_resolve(
        server: &MockServer,
        query: &str,
        response: Value,
    ) -> Result<Value, Error> {
        use dynaql_parser::parse_query;
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, ResponseTemplate};

        Mock::given(method("POST"))
            .and(path("/"))
            .and(header("User-Agent", "Grafbase"))
            .and(header("Authorization", "Bearer FOOBAR"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response.clone()))
            .expect(1)
            .mount(&server)
            .await;

        let resolver = Resolver {
            api_name: "myApi".to_owned(),
            url: Url::parse(&server.uri()).unwrap(),
        };

        let headers = vec![("Authorization".to_string(), "Bearer FOOBAR".to_string())];
        let document = parse_query(query).unwrap();

        let fragment_definitions = document
            .fragments
            .iter()
            .map(|(k, v)| (k, v.as_ref().node))
            .collect();

        let operation = document
            .operations
            .iter()
            .next()
            .unwrap()
            .1
            .clone()
            .into_inner();

        let selection_set = operation
            .selection_set
            .node
            .items
            .as_slice()
            .iter()
            .map(|v| &v.node);

        let mut errors: Vec<ServerError> = vec![];
        let error_handler = |error| errors.push(error);

        let value = resolver
            .resolve(
                OperationType::Query,
                &headers,
                fragment_definitions,
                selection_set,
                error_handler,
            )
            .await?
            .data_resolved;

        let data = Arc::try_unwrap(value).unwrap();
        let response = if errors.is_empty() {
            json!({ "data": data })
        } else {
            json!({ "data": data, "errors": errors })
        };

        Ok(response)
    }
}
