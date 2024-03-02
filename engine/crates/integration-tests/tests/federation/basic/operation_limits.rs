use engine_v2::Engine;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{federation::EngineV2Ext, runtime};

#[rstest::rstest]
#[case( // 1
    "@operationLimits(depth: 1)",
    r#"query {
        allBotPullRequests {
            title
        }
    }"#,
    Some("Query is nested too deep.")
)]
#[case( // 2
   "@operationLimits(depth: 2)",
    r#"query {
        allBotPullRequests {
            title
        }
    }"#,
    None
)]
#[case( // 3
    "@operationLimits(height: 1)",
    r#"query {
        favoriteRepository
        serverVersion
    }"#,
    Some("Query is too high.")
)]
#[case( // 4
    "@operationLimits(height: 2)",
    r#"query {
        favoriteRepository
        serverVersion
    }"#,
    None
)]
#[case( // 5
    "@operationLimits(height: 2)",
    r#"query {
        favoriteRepository
        serverVersion
        aliasedRepeateDoesNotCount: serverVersion
    }"#,
    None
)]
#[case( // 6
    "@operationLimits(complexity: 1)",
    r#"query {
        favoriteRepository
        serverVersion
    }"#,
    Some("Query is too complex.")
)]
#[case( // 7
    "@operationLimits(complexity: 2)",
    r#"query {
        favoriteRepository
        serverVersion
    }"#,
    None
)]
#[case( // 8
    "@operationLimits(complexity: 2)",
    r#"query {
        favoriteRepository
        serverVersion
        aliasedRepeateDoesCount: serverVersion
    }"#,
    Some("Query is too complex.")
)]
#[case( // 9
    "@operationLimits(complexity: 3)",
    r#"query {
        favoriteRepository
        allBotPullRequests {
            title
            aliasedRepeateDoesCount: title
        }
    }"#,
    Some("Query is too complex.")
)]
#[case( // 10
    "@operationLimits(aliases: 2)",
    r#"query {
        favoriteRepository
        favorite: favoriteRepository
        allBotPullRequests {
            title
            aliasedRepeateDoesCount: title
        }
    }"#,
    None
)]
#[case( // 11
    "@operationLimits(aliases: 1)",
    r#"query {
        favoriteRepository
        favorite: favoriteRepository
        allBotPullRequests {
            title
            aliasedRepeateDoesCount: title
        }
    }"#,
    Some("Query contains too many aliases.")
)]
#[case( // 12
    "@operationLimits(rootFields: 2)",
    r#"query {
        favoriteRepository
        serverVersion
        aliasedRepeateDoesCount: serverVersion
    }"#,
    Some("Query contains too many root fields.")
)]
#[case( // 13
    "@operationLimits(rootFields: 3)",
    r#"query {
        favoriteRepository
        serverVersion
        aliasedRepeateDoesCount: serverVersion
    }"#,
    None
)]
fn test_operation_limits(
    #[case] operation_limits_config: &'static str,
    #[case] query: &'static str,
    #[case] error: Option<&'static str>,
) {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_supergraph_config(format!("extend schema {operation_limits_config}"))
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

        engine.execute(query).await
    });

    assert_eq!(
        response
            .errors()
            .iter()
            .map(|error| error
                .as_object()
                .expect("errors are objects")
                .get("message")
                .and_then(|property| property.as_str())
                .expect("errors must have a `message` string property")
                .to_owned())
            .collect::<Vec<_>>(),
        error.into_iter().map(str::to_owned).collect::<Vec<_>>()
    );
}
