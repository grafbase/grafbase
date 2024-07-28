use async_graphql::{
    scalar, ComplexObject, EmptyMutation, EmptySubscription, InputObject, Interface, Object, SimpleObject, Union, ID,
};

use crate::{MockGraphQlServer, Subgraph};

pub struct FakeGithubSchema;

impl Subgraph for FakeGithubSchema {
    fn name(&self) -> String {
        "github".to_string()
    }

    async fn start(self) -> MockGraphQlServer {
        MockGraphQlServer::new(self).await
    }
}

#[async_trait::async_trait]
impl super::Schema for FakeGithubSchema {
    async fn execute(
        &self,
        _headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response {
        async_graphql::Schema::build(Query, EmptyMutation, EmptySubscription)
            .finish()
            .execute(request)
            .await
    }

    fn execute_stream(
        &self,
        request: async_graphql::Request,
    ) -> futures::stream::BoxStream<'static, async_graphql::Response> {
        Box::pin(
            async_graphql::Schema::build(Query, EmptyMutation, EmptySubscription)
                .finish()
                .execute_stream(request),
        )
    }

    fn sdl(&self) -> String {
        let schema = async_graphql::Schema::build(Query, EmptyMutation, EmptySubscription).finish();

        schema.sdl_with_options(async_graphql::SDLExportOptions::new())
    }
}

struct Query;

#[derive(serde::Serialize, serde::Deserialize)]
struct CustomRepoId {
    owner: String,
    name: String,
}

scalar!(CustomRepoId);

#[Object]
impl Query {
    async fn favorite_repository(&self) -> CustomRepoId {
        CustomRepoId {
            owner: "rust-lang".to_string(),
            name: "rust".to_string(),
        }
    }

    // A top level scalar field for testing
    async fn server_version(&self) -> &str {
        "1"
    }

    async fn pull_requests_and_issues(&self, _filter: PullRequestsAndIssuesFilters) -> Vec<PullRequestOrIssue> {
        // This doesn't actually filter anything because I don't need that for my test.
        vec![
            PullRequestOrIssue::PullRequest(PullRequest {
                id: "1".into(),
                title: "Creating the thing".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::User(User {
                    name: "Jim".into(),
                    email: "jim@example.com".into(),
                }),
                status: Status::Open,
            }),
            PullRequestOrIssue::PullRequest(PullRequest {
                id: "2".into(),
                title: "Some bot PR".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::Bot(Bot { id: "123".into() }),
                status: Status::Closed,
            }),
            PullRequestOrIssue::Issue(Issue {
                title: "Everythings fine".into(),
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
                id: "1".into(),
                title: "Creating the thing".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::User(User {
                    name: "Jim".into(),
                    email: "jim@example.com".into(),
                }),
                status: Status::Open,
            },
            PullRequest {
                id: "2".into(),
                title: "Some bot PR".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::Bot(Bot { id: "123".into() }),
                status: Status::Closed,
            },
        ]
    }

    async fn all_bot_pull_requests(&self) -> Vec<PullRequest> {
        vec![
            PullRequest {
                id: "1".into(),
                title: "Creating the thing".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::User(User {
                    name: "Jim".into(),
                    email: "jim@example.com".into(),
                }),
                status: Status::Open,
            },
            PullRequest {
                id: "2".into(),
                title: "Some bot PR".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::Bot(Bot { id: "123".into() }),
                status: Status::Closed,
            },
        ]
    }

    async fn pull_request(&self, id: ID) -> Option<PullRequest> {
        if id == "1" {
            return Some(PullRequest {
                id: "1".into(),
                title: "Creating the thing".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::User(User {
                    name: "Jim".into(),
                    email: "jim@example.com".into(),
                }),
                status: Status::Open,
            });
        } else if id == "2" {
            return Some(PullRequest {
                id: "2".into(),
                title: "Some bot PR".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::Bot(Bot { id: "123".into() }),
                status: Status::Closed,
            });
        }
        None
    }

    async fn pull_request_or_issue(&self, id: ID) -> Option<PullRequestOrIssue> {
        if id == "1" {
            return Some(PullRequestOrIssue::PullRequest(PullRequest {
                id: "1".into(),
                title: "Creating the thing".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::User(User {
                    name: "Jim".into(),
                    email: "jim@example.com".into(),
                }),
                status: Status::Closed,
            }));
        } else if id == "2" {
            return Some(PullRequestOrIssue::PullRequest(PullRequest {
                id: "2".into(),
                title: "Some bot PR".into(),
                checks: vec!["Success!".into()],
                author: UserOrBot::Bot(Bot { id: "123".into() }),
                status: Status::Closed,
            }));
        } else if id == "3" {
            return Some(PullRequestOrIssue::Issue(Issue {
                title: "Everythings fine".into(),
                author: UserOrBot::User(User {
                    name: "The Pessimist".into(),
                    email: "pessimist@example.com".into(),
                }),
            }));
        }
        None
    }

    async fn status_string(&self, status: Status) -> &str {
        match status {
            Status::Open => "boo its closed",
            Status::Closed => "woo its open",
        }
    }
}

#[derive(SimpleObject)]
struct PullRequest {
    id: async_graphql::ID,
    title: String,
    checks: Vec<String>,
    author: UserOrBot,
    status: Status,
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
#[graphql(complex)]
struct User {
    name: String,
    email: String,
}

#[ComplexObject]
impl User {
    async fn pull_requests(&self) -> Vec<PullRequest> {
        vec![]
    }
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

#[derive(async_graphql::Enum, Clone, Copy, Eq, PartialEq)]
enum Status {
    Open,
    Closed,
}
