use async_graphql::{EmptyMutation, EmptySubscription, InputObject, Interface, Object, SimpleObject, Union, ID};

pub struct FakeGithubSchema;

#[async_trait::async_trait]
impl super::Schema for FakeGithubSchema {
    async fn execute(
        &self,
        headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response {
        async_graphql::Schema::build(Query { headers }, EmptyMutation, EmptySubscription)
            .finish()
            .execute(request)
            .await
    }

    fn sdl(&self) -> String {
        let schema = async_graphql::Schema::build(
            Query {
                headers: Default::default(),
            },
            EmptyMutation,
            EmptySubscription,
        )
        .finish();

        schema.sdl_with_options(async_graphql::SDLExportOptions::new())
    }
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
