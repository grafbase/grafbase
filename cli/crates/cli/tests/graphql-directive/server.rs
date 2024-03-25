use std::time::Duration;

use async_graphql::{EmptyMutation, EmptySubscription, Interface, Object, Schema, SimpleObject, Union, ID};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{http::HeaderMap, routing::post, Router};

async fn graphql_handler(headers: HeaderMap, req: GraphQLRequest) -> GraphQLResponse {
    let headers = headers
        .into_iter()
        .map(|(name, value)| {
            (
                name.map(|name| name.to_string()).unwrap_or_default(),
                String::from_utf8_lossy(value.as_bytes()).to_string(),
            )
        })
        .collect();
    let schema = Schema::build(Query { headers }, EmptyMutation, EmptySubscription).finish();

    schema.execute(req.into_inner()).await.into()
}

pub(crate) async fn run() -> u16 {
    let app = Router::new().route("/", post(graphql_handler));

    let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = tcp_listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(tcp_listener, app).await.unwrap();
    });

    // Give the server time to start
    tokio::time::sleep(Duration::from_millis(20)).await;

    port
}

struct Query {
    headers: Vec<(String, String)>,
}

#[Object]
impl Query {
    // A top level scalar field for testing
    async fn server_version(&self) -> &str {
        "1"
    }

    async fn pull_request_or_issue(&self, id: ID) -> Option<PullRequestOrIssue> {
        if id == "1" {
            return Some(PullRequestOrIssue::PullRequest(PullRequest {
                title: "Creating the thing".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::User(User {
                    name: "Jim".into(),
                    email: "jim@example.com".into(),
                }),
            }));
        } else if id == "2" {
            return Some(PullRequestOrIssue::PullRequest(PullRequest {
                title: "Some bot PR".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::Bot(Bot { id: "123".into() }),
            }));
        } else if id == "3" {
            return Some(PullRequestOrIssue::Issue(Issue {
                title: "Everything's fine".into(),
                author: UserOrBot::User(User {
                    name: "The Pessimist".into(),
                    email: "pessimist@example.com".into(),
                }),
            }));
        }
        None
    }

    async fn headers(&self) -> Vec<Header> {
        self.headers
            .clone()
            .into_iter()
            .map(|(name, value)| Header { name, value })
            .collect()
    }
}

#[derive(SimpleObject)]
struct Header {
    name: String,
    value: String,
}

#[derive(SimpleObject)]
struct PullRequest {
    title: String,
    checks: Vec<String>,
    author: UserOrBot,
}

#[derive(SimpleObject)]
struct Issue {
    title: String,
    author: UserOrBot,
}

#[derive(Interface)]
#[graphql(field(name = "title", ty = "String"), field(name = "author", ty = "UserOrBot"))]
enum PullRequestOrIssue {
    PullRequest(PullRequest),
    Issue(Issue),
}

#[derive(Union, Clone)]
enum UserOrBot {
    User(User),
    Bot(Bot),
}

#[derive(SimpleObject, Clone)]
struct User {
    name: String,
    email: String,
}

#[derive(SimpleObject, Clone)]
struct Bot {
    id: ID,
}

impl From<&UserOrBot> for UserOrBot {
    fn from(value: &UserOrBot) -> Self {
        value.clone()
    }
}
