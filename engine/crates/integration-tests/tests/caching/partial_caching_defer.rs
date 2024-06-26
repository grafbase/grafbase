use integration_tests::{
    runtime,
    udfs::{RustResolver, RustUdfs},
    EngineBuilder,
};
use runtime::udf::UdfResponse;
use serde_json::json;

#[test]
fn smoke_test() {
    const SCHEMA: &str = r#"
    extend schema @experimental(partialCaching: true)

    type Query {
        user: User @resolver(name: "user")
    }

    type User {
        name: String @cache(maxAge: 140)
        email: String @cache(maxAge: 130)
        someConstant: String @cache(maxAge: 120)
        uncached: String
    }
    "#;

    runtime().block_on(async {
        let gateway = EngineBuilder::new(SCHEMA)
            .with_custom_resolvers(RustUdfs::new().resolver("user", UserResolver::default()))
            .gateway_builder()
            .await
            .build();

        const QUERY: &str = r#"
            query {
                user {
                    name
                    ... @defer(label: "woo") {
                        email
                        someConstant
                    }
                }
            }
        "#;

        let responses = gateway.execute(QUERY).collect().await;

        insta::assert_json_snapshot!(responses, @r###"
        [
          {
            "data": {
              "user": {
                "name": "Jo 1"
              }
            },
            "hasNext": true
          },
          {
            "data": {
              "name": "Jo 1",
              "email": "1@example.com",
              "someConstant": "blah 1"
            },
            "path": [
              "user"
            ],
            "hasNext": false,
            "label": "woo"
          }
        ]
        "###);

        // Call it again and see what has been cached/not
        let responses = gateway.execute(QUERY).collect().await;

        insta::assert_json_snapshot!(responses, @r###"
        [
          {
            "data": {
              "user": {
                "name": "Jo 1",
                "email": "1@example.com",
                "someConstant": "blah 1"
              }
            },
            "hasNext": false
          }
        ]
        "###);
    });
}

#[derive(Default)]
pub struct UserResolver {
    call_count: usize,
}

impl RustResolver for UserResolver {
    fn invoke(
        &mut self,
        _payload: runtime::udf::CustomResolverRequestPayload,
    ) -> Result<UdfResponse, runtime::udf::UdfError> {
        self.call_count += 1;
        let call_count = self.call_count;

        let name = format!("Jo {call_count}");
        let email = format!("{call_count}@example.com");
        let constant = format!("blah {call_count}");
        let uncached = format!("dont cache me bro {call_count}");

        Ok(UdfResponse::Success(json!({
            "name": name,
            "email": email,
            "someConstant": constant,
            "uncached": uncached
        })))
    }
}
