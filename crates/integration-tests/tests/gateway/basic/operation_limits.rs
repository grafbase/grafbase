use graphql_mocks::FakeGithubSchema;
use integration_tests::{gateway::Gateway, runtime};

#[rstest::rstest]
#[case( // 1
    r#"
        [operation_limits]
        depth = 1
    "#,
    r#"query {
        allBotPullRequests {
            title
        }
    }"#,
    Some("Query is nested too deep.")
)]
#[case( // 2
    r#"
        [operation_limits]
        depth = 2
    "#,
    r#"query {
        allBotPullRequests {
            title
        }
    }"#,
    None
)]
#[case( // 3
    r#"
        [operation_limits]
        height = 1
    "#,
    r#"query {
        favoriteRepository
        serverVersion
    }"#,
    Some("Query is too high.")
)]
#[case( // 4
    r#"
        [operation_limits]
        height = 2
    "#,
    r#"query {
        favoriteRepository
        serverVersion
    }"#,
    None
)]
#[case( // 5
    r#"
        [operation_limits]
        height = 2
    "#,
    r#"query {
        favoriteRepository
        serverVersion
        aliasedRepeateDoesNotCount: serverVersion
    }"#,
    None
)]
#[case( // 6
    r#"
        [operation_limits]
        complexity = 1
    "#,
    r#"query {
        favoriteRepository
        serverVersion
    }"#,
    Some("Query is too complex.")
)]
#[case( // 7
    r#"
        [operation_limits]
        complexity = 2
    "#,
    r#"query {
        favoriteRepository
        serverVersion
    }"#,
    None
)]
#[case( // 8
    r#"
        [operation_limits]
        complexity = 2
    "#,
    r#"query {
        favoriteRepository
        serverVersion
        aliasedRepeateDoesCount: serverVersion
    }"#,
    Some("Query is too complex.")
)]
#[case( // 9
    r#"
        [operation_limits]
        complexity = 3
    "#,
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
    r#"
        [operation_limits]
        aliases = 2
    "#,
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
    r#"
        [operation_limits]
        aliases = 1
    "#,
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
    r#"
        [operation_limits]
        root_fields = 2
    "#,
    r#"query {
        favoriteRepository
        serverVersion
        aliasedRepeateDoesCount: serverVersion
    }"#,
    Some("Query contains too many root fields.")
)]
#[case( // 13
    r#"
        [operation_limits]
        root_fields = 3
    "#,
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
        let engine = Gateway::builder()
            .with_toml_config(operation_limits_config)
            .with_subgraph(FakeGithubSchema)
            .build()
            .await;

        engine.post(query).await
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
