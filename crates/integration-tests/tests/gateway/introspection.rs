use cynic::{QueryBuilder, http::ReqwestExt};
use cynic_introspection::{CapabilitiesQuery, IntrospectionQuery, SpecificationVersion};
use graphql_mocks::{
    EchoSchema, FakeGithubSchema, FederatedAccountsSchema, FederatedInventorySchema, FederatedProductsSchema,
    FederatedReviewsSchema,
};
use indoc::indoc;
use integration_tests::{gateway::Gateway, runtime};

pub const PATHFINDER_INTROSPECTION_QUERY: &str = include_str!("../../data/introspection.graphql");

pub const CONFIG: &str = indoc! {r#"
    [graph]
    introspection = true
"#};

#[test]
fn can_run_pathfinder_introspection_query() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .with_toml_config(CONFIG)
            .build()
            .await;

        engine.post(PATHFINDER_INTROSPECTION_QUERY).await
    });
    assert!(response.errors().is_empty(), "{response}");

    insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r###"
    type Bot {
      id: ID!
    }

    input BotInput {
      id: ID!
      sentient: Boolean! = false
    }

    scalar CustomRepoId

    type Issue implements PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    type PullRequest implements PullRequestOrIssue {
      author: UserOrBot!
      checks: [String!]!
      id: ID!
      status: Status!
      title: String!
    }

    interface PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    input PullRequestsAndIssuesFilters {
      search: String!
    }

    type Query {
      allBotPullRequests: [PullRequest!]!
      botPullRequests(bots: [[BotInput!]]!): [PullRequest!]!
      fail: Int!
      favoriteRepository: CustomRepoId!
      pullRequest(id: ID!): PullRequest
      pullRequestOrIssue(id: ID!): PullRequestOrIssue
      pullRequestsAndIssues(filter: PullRequestsAndIssuesFilters!): [PullRequestOrIssue!]!
      serverVersion: String!
      sillyDefaultValue(status: Status! = OPEN): String!
      statusString(status: Status!): String!
    }

    enum Status {
      OPEN
      CLOSED
    }

    type User {
      email: String!
      name: String!
      pullRequests: [PullRequest!]!
    }

    union UserOrBot = Bot | User

    "###);
}

#[test]
fn can_run_2018_introspection_query() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .with_toml_config(CONFIG)
            .build()
            .await;

        engine
            .post(IntrospectionQuery::with_capabilities(
                SpecificationVersion::June2018.capabilities(),
            ))
            .await
    });
    assert!(response.errors().is_empty(), "{response}");

    insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r###"
    type Bot {
      id: ID!
    }

    input BotInput {
      id: ID!
      sentient: Boolean! = false
    }

    scalar CustomRepoId

    type Issue implements PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    type PullRequest implements PullRequestOrIssue {
      author: UserOrBot!
      checks: [String!]!
      id: ID!
      status: Status!
      title: String!
    }

    interface PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    input PullRequestsAndIssuesFilters {
      search: String!
    }

    type Query {
      allBotPullRequests: [PullRequest!]!
      botPullRequests(bots: [[BotInput!]]!): [PullRequest!]!
      fail: Int!
      favoriteRepository: CustomRepoId!
      pullRequest(id: ID!): PullRequest
      pullRequestOrIssue(id: ID!): PullRequestOrIssue
      pullRequestsAndIssues(filter: PullRequestsAndIssuesFilters!): [PullRequestOrIssue!]!
      serverVersion: String!
      sillyDefaultValue(status: Status! = OPEN): String!
      statusString(status: Status!): String!
    }

    enum Status {
      OPEN
      CLOSED
    }

    type User {
      email: String!
      name: String!
      pullRequests: [PullRequest!]!
    }

    union UserOrBot = Bot | User

    "###);
}

#[test]
fn can_run_2021_introspection_query() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .with_toml_config(CONFIG)
            .build()
            .await;

        engine
            .post(IntrospectionQuery::with_capabilities(
                SpecificationVersion::October2021.capabilities(),
            ))
            .await
    });
    assert!(response.errors().is_empty(), "{response}");

    insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r###"
    type Bot {
      id: ID!
    }

    input BotInput {
      id: ID!
      sentient: Boolean! = false
    }

    scalar CustomRepoId

    type Issue implements PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    type PullRequest implements PullRequestOrIssue {
      author: UserOrBot!
      checks: [String!]!
      id: ID!
      status: Status!
      title: String!
    }

    interface PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    input PullRequestsAndIssuesFilters {
      search: String!
    }

    type Query {
      allBotPullRequests: [PullRequest!]!
      botPullRequests(bots: [[BotInput!]]!): [PullRequest!]!
      fail: Int!
      favoriteRepository: CustomRepoId!
      pullRequest(id: ID!): PullRequest
      pullRequestOrIssue(id: ID!): PullRequestOrIssue
      pullRequestsAndIssues(filter: PullRequestsAndIssuesFilters!): [PullRequestOrIssue!]!
      serverVersion: String!
      sillyDefaultValue(status: Status! = OPEN): String!
      statusString(status: Status!): String!
    }

    enum Status {
      OPEN
      CLOSED
    }

    type User {
      email: String!
      name: String!
      pullRequests: [PullRequest!]!
    }

    union UserOrBot = Bot | User

    "###);
}

#[test]
fn echo_subgraph_introspection() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(EchoSchema::default())
            .with_toml_config(CONFIG)
            .build()
            .await;

        engine
            .post(IntrospectionQuery::with_capabilities(
                SpecificationVersion::October2021.capabilities(),
            ))
            .await
    });
    assert!(response.errors().is_empty(), "{response}");

    insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r#"
    enum FancyBool {
      YES
      NO
    }

    type Header {
      name: String!
      value: String!
    }

    input InputObj {
      string: String
      int: Int
      float: Float
      id: ID
      annoyinglyOptionalStrings: [[String]]
      recursiveObject: InputObj
      recursiveObjectList: [InputObj!]
      fancyBool: FancyBool
    }

    """
    A scalar that can represent any JSON value.
    """
    scalar JSON

    type Query {
      fancyBool(input: FancyBool!): FancyBool!
      float(input: Float!): Float!
      header(name: String!): String
      headers: [Header!]!
      id(input: ID!): ID!
      inputObject(input: InputObj!): JSON
      int(input: Int!): Int!
      listOfInputObject(input: InputObj!): JSON!
      listOfListOfStrings(input: [[String!]!]!): [[String!]!]!
      listOfStrings(input: [String!]!): [String!]!
      optionalListOfOptionalStrings(input: [String]): [String]
      responseHeader(name: String!, value: String!): Boolean
      string(input: String!): String!
    }
    "#);
}

#[test]
fn can_run_capability_introspection_query() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .with_toml_config(CONFIG)
            .build()
            .await;

        engine.post(CapabilitiesQuery::build(())).await
    });
    assert!(response.errors().is_empty(), "{response}");

    let response = serde_json::from_value::<CapabilitiesQuery>(response.into_data()).expect("valid response");

    assert_eq!(
        response.capabilities().version_supported(),
        SpecificationVersion::September2025
    );
}

#[test]
#[ignore]
#[allow(clippy::panic)]
fn introspection_output_matches_source() {
    use reqwest::Client;
    let (response, _upstream_sdl) = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .with_toml_config(CONFIG)
            .build()
            .await;

        let response = engine.post(IntrospectionQuery::build(())).await;

        let upstream_sdl = Client::new()
            .post(engine.subgraph::<FakeGithubSchema>().url())
            .run_graphql(IntrospectionQuery::build(()))
            .await
            .expect("request to work")
            .data
            .expect("data to be present")
            .into_schema()
            .expect("valid schema")
            .to_sdl();

        (response, upstream_sdl)
    });
    assert!(response.errors().is_empty(), "{response}");

    let _engine_sdl = introspection_to_sdl(response.into_data());

    panic!("How to compare efficiently to DSL? They don't have the same ordering of fields or types.");
}

#[test]
fn raw_introspection_output() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .with_subgraph(EchoSchema::default())
            .with_toml_config(CONFIG)
            .build()
            .await;

        engine.post(IntrospectionQuery::build(())).await
    });

    // Some errors are just easier to understand with the actual introspection output.
    insta::assert_json_snapshot!(response);
}

#[test]
fn can_introsect_when_multiple_subgraphs() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .with_subgraph(EchoSchema::default())
            .with_toml_config(CONFIG)
            .build()
            .await;

        engine.post(IntrospectionQuery::build(())).await
    });
    assert!(response.errors().is_empty(), "{response}");

    insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r#"
    type Bot {
      id: ID!
    }

    input BotInput {
      id: ID!
      sentient: Boolean! = false
    }

    scalar CustomRepoId

    enum FancyBool {
      YES
      NO
    }

    type Header {
      name: String!
      value: String!
    }

    input InputObj {
      string: String
      int: Int
      float: Float
      id: ID
      annoyinglyOptionalStrings: [[String]]
      recursiveObject: InputObj
      recursiveObjectList: [InputObj!]
      fancyBool: FancyBool
    }

    type Issue implements PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    """
    A scalar that can represent any JSON value.
    """
    scalar JSON

    type PullRequest implements PullRequestOrIssue {
      author: UserOrBot!
      checks: [String!]!
      id: ID!
      status: Status!
      title: String!
    }

    interface PullRequestOrIssue {
      author: UserOrBot!
      title: String!
    }

    input PullRequestsAndIssuesFilters {
      search: String!
    }

    type Query {
      allBotPullRequests: [PullRequest!]!
      botPullRequests(bots: [[BotInput!]]!): [PullRequest!]!
      fail: Int!
      fancyBool(input: FancyBool!): FancyBool!
      favoriteRepository: CustomRepoId!
      float(input: Float!): Float!
      header(name: String!): String
      headers: [Header!]!
      id(input: ID!): ID!
      inputObject(input: InputObj!): JSON
      int(input: Int!): Int!
      listOfInputObject(input: InputObj!): JSON!
      listOfListOfStrings(input: [[String!]!]!): [[String!]!]!
      listOfStrings(input: [String!]!): [String!]!
      optionalListOfOptionalStrings(input: [String]): [String]
      pullRequest(id: ID!): PullRequest
      pullRequestOrIssue(id: ID!): PullRequestOrIssue
      pullRequestsAndIssues(filter: PullRequestsAndIssuesFilters!): [PullRequestOrIssue!]!
      responseHeader(name: String!, value: String!): Boolean
      serverVersion: String!
      sillyDefaultValue(status: Status! = OPEN): String!
      statusString(status: Status!): String!
      string(input: String!): String!
    }

    enum Status {
      OPEN
      CLOSED
    }

    type User {
      email: String!
      name: String!
      pullRequests: [PullRequest!]!
    }

    union UserOrBot = Bot | User
    "#);
}

#[test]
fn supports_the_type_field() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .with_toml_config(CONFIG)
            .build()
            .await;

        engine
            .post(
                r#"
                    query {
                        __type(name: "PullRequest") {
                            kind
                            name
                            description
                            fields(includeDeprecated: true) {
                                name
                            }
                            interfaces {
                                name
                            }
                            possibleTypes {
                                name
                            }
                            enumValues {
                                name
                            }
                            inputFields {
                                name
                            }
                            ofType {
                                kind
                                name
                            }
                        }
                    }
                    "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "__type": {
          "kind": "OBJECT",
          "name": "PullRequest",
          "description": null,
          "fields": [
            {
              "name": "author"
            },
            {
              "name": "checks"
            },
            {
              "name": "id"
            },
            {
              "name": "status"
            },
            {
              "name": "title"
            }
          ],
          "interfaces": [
            {
              "name": "PullRequestOrIssue"
            }
          ],
          "possibleTypes": null,
          "enumValues": null,
          "inputFields": null,
          "ofType": null
        }
      }
    }
    "###);
}

#[test]
fn type_field_returns_null_on_missing_type() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(CONFIG)
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

        engine
            .post(
                r#"
                    query {
                        __type(name: "Boom") {
                            kind
                            name
                        }
                    }
                    "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "__type": null
      }
    }
    "###);
}

#[test]
fn supports_recursing_through_types() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(CONFIG)
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

        engine
            .post(
                r#"
                    query {
                        __type(name: "PullRequestOrIssue") {
                            possibleTypes {
                                name
                                interfaces {
                                    name
                                    possibleTypes {
                                        name
                                        interfaces {
                                            name
                                            possibleTypes {
                                                name
                                                interfaces {
                                                    name
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "__type": {
          "possibleTypes": [
            {
              "name": "Issue",
              "interfaces": [
                {
                  "name": "PullRequestOrIssue",
                  "possibleTypes": [
                    {
                      "name": "Issue",
                      "interfaces": [
                        {
                          "name": "PullRequestOrIssue",
                          "possibleTypes": [
                            {
                              "name": "Issue",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            },
                            {
                              "name": "PullRequest",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "PullRequest",
                      "interfaces": [
                        {
                          "name": "PullRequestOrIssue",
                          "possibleTypes": [
                            {
                              "name": "Issue",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            },
                            {
                              "name": "PullRequest",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            },
            {
              "name": "PullRequest",
              "interfaces": [
                {
                  "name": "PullRequestOrIssue",
                  "possibleTypes": [
                    {
                      "name": "Issue",
                      "interfaces": [
                        {
                          "name": "PullRequestOrIssue",
                          "possibleTypes": [
                            {
                              "name": "Issue",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            },
                            {
                              "name": "PullRequest",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    },
                    {
                      "name": "PullRequest",
                      "interfaces": [
                        {
                          "name": "PullRequestOrIssue",
                          "possibleTypes": [
                            {
                              "name": "Issue",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            },
                            {
                              "name": "PullRequest",
                              "interfaces": [
                                {
                                  "name": "PullRequestOrIssue"
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        }
      }
    }
    "###);
}

#[test]
fn rejects_bogus_introspection_queries() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(CONFIG)
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

        engine
            .post(
                r#"
                    query {
                        __type(name: "PullRequestOrIssue") {
                            possibleTypes {
                                blarg
                            }
                        }
                    }
                    "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "errors": [
        {
          "message": "__Type does not have a field named 'blarg'.",
          "locations": [
            {
              "line": 5,
              "column": 33
            }
          ],
          "extensions": {
            "code": "OPERATION_VALIDATION_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn introspection_on_multiple_federation_subgraphs() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(CONFIG)
            .with_subgraph(FederatedAccountsSchema::default())
            .with_subgraph(FederatedProductsSchema::default())
            .with_subgraph(FederatedReviewsSchema::default())
            .with_subgraph(FederatedInventorySchema::default())
            .build()
            .await;

        engine.post(PATHFINDER_INTROSPECTION_QUERY).await
    });
    assert!(response.errors().is_empty(), "{response}");

    insta::assert_snapshot!(introspection_to_sdl(response.into_data()), @r#"
    type BusinessAccount {
      businessName: String!
      email: String!
      id: ID!
      joinedTimestamp: Int!
    }

    type Cart {
      products: [Product!]!
    }

    type DeliveryCompany implements ShippingService {
      companyType: String!
      id: String!
      name: String!
      reviews: [ShippingServiceReview!]!
    }

    type HomingPigeon implements ShippingService {
      id: String!
      name: String!
      nickname: String!
      reviews: [ShippingServiceReview!]!
    }

    """
    A scalar that can represent any JSON value.
    """
    scalar JSON

    type Picture {
      height: Int!
      url: String!
      width: Int!
    }

    type Product {
      availableShippingService: [ShippingService!]!
      name: String!
      price: Int!
      reviews: [Review!]!
      shippingEstimate: Int!
      upc: String!
      weight(unit: WeightUnit!): Float!
    }

    type Query {
      me: User!
      product(upc: String!): Product
      topProducts: [Product!]!
    }

    type Review {
      author: User
      body: String!
      id: ID!
      pictures: [Picture!]!
      product: Product!
    }

    interface ShippingService {
      id: String!
      name: String!
      reviews: [ShippingServiceReview!]!
    }

    type ShippingServiceReview {
      body: String!
    }

    type Subscription {
      connectionInitPayload: JSON
      httpHeader(name: [String!]!): JSON!
      newProducts: Product!
    }

    enum Trustworthiness {
      REALLY_TRUSTED
      KINDA_TRUSTED
      NOT_TRUSTED
    }

    type User {
      cart: Cart!
      id: ID!
      joinedTimestamp: Int!
      profilePicture: Picture
      """
      This used to be part of this subgraph, but is now being overridden from
      `reviews`
      """
      reviewCount: Int!
      reviews: [Review!]!
      trustworthiness: Trustworthiness!
      username: String!
    }

    enum WeightUnit {
      KILOGRAM
      GRAM
    }
    "#)
}

#[test]
fn default_values() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(CONFIG)
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

        engine
            .post(
                r#"
                    query {
                        __type(name: "Query") {
                            kind
                            name
                            fields {
                                name
                                args {
                                    name
                                    defaultValue
                                }
                            }
                        }
                    }
                    "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "__type": {
          "kind": "OBJECT",
          "name": "Query",
          "fields": [
            {
              "name": "allBotPullRequests",
              "args": []
            },
            {
              "name": "botPullRequests",
              "args": [
                {
                  "name": "bots",
                  "defaultValue": null
                }
              ]
            },
            {
              "name": "fail",
              "args": []
            },
            {
              "name": "favoriteRepository",
              "args": []
            },
            {
              "name": "pullRequest",
              "args": [
                {
                  "name": "id",
                  "defaultValue": null
                }
              ]
            },
            {
              "name": "pullRequestOrIssue",
              "args": [
                {
                  "name": "id",
                  "defaultValue": null
                }
              ]
            },
            {
              "name": "pullRequestsAndIssues",
              "args": [
                {
                  "name": "filter",
                  "defaultValue": null
                }
              ]
            },
            {
              "name": "serverVersion",
              "args": []
            },
            {
              "name": "sillyDefaultValue",
              "args": [
                {
                  "name": "status",
                  "defaultValue": "OPEN"
                }
              ]
            },
            {
              "name": "statusString",
              "args": [
                {
                  "name": "status",
                  "defaultValue": null
                }
              ]
            }
          ]
        }
      }
    }
    "###);
}

#[allow(clippy::panic)]
pub fn introspection_to_sdl(data: serde_json::Value) -> String {
    serde_json::from_value::<IntrospectionQuery>(data)
        .expect("valid response")
        .into_schema()
        .expect("valid schema")
        .to_sdl()
}

#[test]
fn type_input_fields_include_deprecated_filter() {
    let schema = r#"
        type Query {
            dummy(t: MyType): String
        }

        input MyType {
            old: String @deprecated(reason: "test")
            new: String
        }
        "#;
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(CONFIG)
            .with_subgraph_sdl("test", schema)
            .build()
            .await;

        engine
            .post(
                r#"
                    query {
                        __schema {
                            types {
                                name
                                withDeprecated: inputFields(includeDeprecated: true) { name }
                                withoutDeprecated: inputFields(includeDeprecated: false) { name }
                                defaultDeprecated: inputFields { name }
                            }
                        }
                    }
                    "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "__schema": {
          "types": [
            {
              "name": "Boolean",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            },
            {
              "name": "Float",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            },
            {
              "name": "ID",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            },
            {
              "name": "Int",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            },
            {
              "name": "MyType",
              "withDeprecated": [
                {
                  "name": "old"
                },
                {
                  "name": "new"
                }
              ],
              "withoutDeprecated": [
                {
                  "name": "new"
                }
              ],
              "defaultDeprecated": [
                {
                  "name": "new"
                }
              ]
            },
            {
              "name": "Query",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            },
            {
              "name": "String",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            },
            {
              "name": "__Directive",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            },
            {
              "name": "__DirectiveLocation",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            },
            {
              "name": "__EnumValue",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            },
            {
              "name": "__Field",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            },
            {
              "name": "__InputValue",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            },
            {
              "name": "__Schema",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            },
            {
              "name": "__Type",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            },
            {
              "name": "__TypeKind",
              "withDeprecated": null,
              "withoutDeprecated": null,
              "defaultDeprecated": null
            }
          ]
        }
      }
    }
    "###);
}

#[test]
fn directive_args_include_deprecated_filter() {
    let schema = r#"
        extend schema
          @link(url: "https://specs.apollo.dev/federation/v2.3", import: ["@composeDirective"])
          @composeDirective(name: "@test")

        type Query {
            test: String
        }

        directive @test(
            new: String
            old: String @deprecated(reason: "test")
        ) on OBJECT
        "#;

    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(CONFIG)
            .with_subgraph_sdl("test", schema)
            .build()
            .await;

        engine
            .post(
                r#"
                    query {
                        __schema {
                            directives {
                                name
                                withDeprecated: args(includeDeprecated: true) { name }
                                withoutDeprecated: args(includeDeprecated: false) { name }
                                defaultDeprecated: args { name }
                            }
                        }
                    }
                    "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "__schema": {
          "directives": [
            {
              "name": "skip",
              "withDeprecated": [
                {
                  "name": "if"
                }
              ],
              "withoutDeprecated": [
                {
                  "name": "if"
                }
              ],
              "defaultDeprecated": [
                {
                  "name": "if"
                }
              ]
            },
            {
              "name": "include",
              "withDeprecated": [
                {
                  "name": "if"
                }
              ],
              "withoutDeprecated": [
                {
                  "name": "if"
                }
              ],
              "defaultDeprecated": [
                {
                  "name": "if"
                }
              ]
            },
            {
              "name": "deprecated",
              "withDeprecated": [
                {
                  "name": "reason"
                }
              ],
              "withoutDeprecated": [
                {
                  "name": "reason"
                }
              ],
              "defaultDeprecated": [
                {
                  "name": "reason"
                }
              ]
            },
            {
              "name": "specifiedBy",
              "withDeprecated": [
                {
                  "name": "url"
                }
              ],
              "withoutDeprecated": [
                {
                  "name": "url"
                }
              ],
              "defaultDeprecated": [
                {
                  "name": "url"
                }
              ]
            }
          ]
        }
      }
    }
    "#);
}

#[test]
fn field_args_include_deprecated_filter() {
    let schema = r#"
        type Query {
            test(
                old: String @deprecated(reason: "test")
                new: String
            ): String
        }

        input InputTest {
            old: String @deprecated(reason: "removed")
            new: String
        }
        "#;
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(CONFIG)
            .with_subgraph_sdl("test", schema)
            .build()
            .await;

        engine
            .post(
                r#"
                    query {
                        __schema {
                            types {
                                name
                                inputFields(includeDeprecated: true) {
                                    name
                                    isDeprecated
                                    deprecationReason
                                }
                                fields {
                                    name
                                    withDeprecated: args(includeDeprecated: true) { name }
                                    withoutDeprecated: args(includeDeprecated: false) { name }
                                    defaultDeprecated: args { name }
                                }
                            }
                        }
                    }
                    "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "__schema": {
          "types": [
            {
              "name": "Boolean",
              "inputFields": null,
              "fields": null
            },
            {
              "name": "Float",
              "inputFields": null,
              "fields": null
            },
            {
              "name": "ID",
              "inputFields": null,
              "fields": null
            },
            {
              "name": "InputTest",
              "inputFields": [
                {
                  "name": "old",
                  "isDeprecated": true,
                  "deprecationReason": "removed"
                },
                {
                  "name": "new",
                  "isDeprecated": false,
                  "deprecationReason": null
                }
              ],
              "fields": null
            },
            {
              "name": "Int",
              "inputFields": null,
              "fields": null
            },
            {
              "name": "Query",
              "inputFields": null,
              "fields": [
                {
                  "name": "test",
                  "withDeprecated": [
                    {
                      "name": "old"
                    },
                    {
                      "name": "new"
                    }
                  ],
                  "withoutDeprecated": [
                    {
                      "name": "new"
                    }
                  ],
                  "defaultDeprecated": [
                    {
                      "name": "new"
                    }
                  ]
                }
              ]
            },
            {
              "name": "String",
              "inputFields": null,
              "fields": null
            },
            {
              "name": "__Directive",
              "inputFields": null,
              "fields": [
                {
                  "name": "name",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "description",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "locations",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "args",
                  "withDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ],
                  "withoutDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ],
                  "defaultDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ]
                },
                {
                  "name": "isRepeatable",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                }
              ]
            },
            {
              "name": "__DirectiveLocation",
              "inputFields": null,
              "fields": null
            },
            {
              "name": "__EnumValue",
              "inputFields": null,
              "fields": [
                {
                  "name": "name",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "description",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "isDeprecated",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "deprecationReason",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                }
              ]
            },
            {
              "name": "__Field",
              "inputFields": null,
              "fields": [
                {
                  "name": "name",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "description",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "args",
                  "withDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ],
                  "withoutDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ],
                  "defaultDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ]
                },
                {
                  "name": "isDeprecated",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "deprecationReason",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "type",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                }
              ]
            },
            {
              "name": "__InputValue",
              "inputFields": null,
              "fields": [
                {
                  "name": "name",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "description",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "defaultValue",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "type",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "isDeprecated",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "deprecationReason",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                }
              ]
            },
            {
              "name": "__Schema",
              "inputFields": null,
              "fields": [
                {
                  "name": "description",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "types",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "queryType",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "mutationType",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "subscriptionType",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "directives",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                }
              ]
            },
            {
              "name": "__Type",
              "inputFields": null,
              "fields": [
                {
                  "name": "kind",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "name",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "description",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "inputFields",
                  "withDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ],
                  "withoutDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ],
                  "defaultDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ]
                },
                {
                  "name": "specifiedByURL",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "fields",
                  "withDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ],
                  "withoutDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ],
                  "defaultDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ]
                },
                {
                  "name": "enumValues",
                  "withDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ],
                  "withoutDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ],
                  "defaultDeprecated": [
                    {
                      "name": "includeDeprecated"
                    }
                  ]
                },
                {
                  "name": "ofType",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "possibleTypes",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                },
                {
                  "name": "interfaces",
                  "withDeprecated": [],
                  "withoutDeprecated": [],
                  "defaultDeprecated": []
                }
              ]
            },
            {
              "name": "__TypeKind",
              "inputFields": null,
              "fields": null
            }
          ]
        }
      }
    }
    "###);
}
