//! A mock GraphQL server for testing the GraphQL connector

use std::{net::TcpListener, time::Duration};

use async_graphql::{
    EmptyMutation, EmptySubscription, InputObject, Interface, Object, Schema, SimpleObject, Union, ID,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{http::HeaderMap, routing::post, Router};

pub struct MockGraphQlServer {
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    port: u16,
}

impl Drop for MockGraphQlServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            shutdown.send(()).ok();
        }
    }
}

impl MockGraphQlServer {
    pub async fn new() -> MockGraphQlServer {
        let app = Router::new().route("/", post(graphql_handler));

        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = socket.local_addr().unwrap().port();

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        tokio::spawn(async move {
            axum::Server::from_tcp(socket)
                .unwrap()
                .serve(app.into_make_service())
                .with_graceful_shutdown(async move {
                    shutdown_rx.await.ok();
                })
                .await
                .unwrap();
        });

        // Give the server time to start
        tokio::time::sleep(Duration::from_millis(20)).await;

        MockGraphQlServer {
            shutdown: Some(shutdown_tx),
            port,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

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

struct Query {
    headers: Vec<(String, String)>,
}

#[Object]
impl Query {
    // A top level scalar field for testing
    async fn server_version(&self) -> &str {
        "1"
    }

    async fn pull_requests_and_issues(&self, _filter: PullRequestsAndIssuesFilters) -> Vec<PullRequestOrIssue> {
        // This doesn't actually filter anything because I don't need that for my test.
        vec![
            PullRequestOrIssue::PullRequest(PullRequest {
                title: "Creating the thing".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::User(User {
                    name: "Jim".into(),
                    email: "jim@example.com".into(),
                }),
            }),
            PullRequestOrIssue::PullRequest(PullRequest {
                title: "Some bot PR".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::Bot(Bot { id: "123".into() }),
            }),
            PullRequestOrIssue::Issue(Issue {
                title: "Everythings fucked".into(),
                author: UserOrBot::User(User {
                    name: "The Pessimist".into(),
                    email: "pessimist@example.com".into(),
                }),
            }),
        ]
    }

    #[allow(unused_variables)]
    async fn bot_pull_requests(&self, bots: Vec<Option<Vec<BotInput>>>) -> Vec<PullRequest> {
        vec![
            PullRequest {
                title: "Creating the thing".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::User(User {
                    name: "Jim".into(),
                    email: "jim@example.com".into(),
                }),
            },
            PullRequest {
                title: "Some bot PR".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::Bot(Bot { id: "123".into() }),
            },
        ]
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
                title: "Everythings fucked".into(),
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

#[derive(InputObject)]
struct BotInput {
    id: ID,
}

impl From<&UserOrBot> for UserOrBot {
    fn from(value: &UserOrBot) -> Self {
        value.clone()
    }
}

#[derive(Debug, InputObject)]
struct PullRequestsAndIssuesFilters {
    search: String,
}
