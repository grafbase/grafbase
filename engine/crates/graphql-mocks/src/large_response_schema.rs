use async_graphql::{
    ComplexObject, EmptyMutation, EmptySubscription, InputObject, Interface, Object, SimpleObject, Union, ID,
};

pub struct LargeResponseSchema;

#[async_trait::async_trait]
impl super::Schema for LargeResponseSchema {
    async fn execute(
        &self,
        _headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response {
        async_graphql::Schema::build(Query::new(), EmptyMutation, EmptySubscription)
            .finish()
            .execute(request)
            .await
    }

    fn execute_stream(
        &self,
        request: async_graphql::Request,
    ) -> futures::stream::BoxStream<'static, async_graphql::Response> {
        Box::pin(
            async_graphql::Schema::build(Query::new(), EmptyMutation, EmptySubscription)
                .finish()
                .execute_stream(request),
        )
    }

    fn sdl(&self) -> String {
        let schema = async_graphql::Schema::build(Query::new(), EmptyMutation, EmptySubscription).finish();

        schema.sdl_with_options(async_graphql::SDLExportOptions::new())
    }
}

struct Query {
    data: Vec<PullRequestOrIssue>,
}

impl Query {
    pub fn new() -> Self {
        Query {
            data: std::iter::repeat_with(|| {
                PullRequestOrIssue::PullRequest(PullRequest {
                    id: "1".into(),
                    title: "Creating the thing".into(),
                    checks: vec!["Success!".into()],
                    author: UserOrBot::User(User {
                        name: "Jim".into(),
                        email: "jim@example.com".into(),
                    }),
                    status: Status::Open,
                })
            })
            .take(1000)
            .collect(),
        }
    }
}

#[Object]
impl Query {
    async fn pull_requests_and_issues(&self) -> &Vec<PullRequestOrIssue> {
        // This doesn't actually filter anything because I don't need that for my test.
        &self.data
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

#[derive(async_graphql::Enum, Clone, Copy, Eq, PartialEq)]
enum Status {
    Open,
    Closed,
}
