#![allow(dead_code)]
use serde_json::json;
use std::{
    thread::sleep,
    time::{Duration, SystemTime},
};

pub struct AsyncClient {
    endpoint: String,
    client: reqwest::Client,
    snapshot: Option<String>,
}

// the query used by https://github.com/graphql/graphql-js/blob/main/src/utilities/getIntrospectionQuery.ts
const INTROSPECTION_QUERY: &str = r#"
query  IntrospectionQuery  {  __schema  {  queryType  {  name  }  mutationType  {  name  }  subscriptionType  {  name  }  types  {  ...FullType  }  directives  {  name  description  locations  args  {  ...InputValue  }  }  }  }  fragment  FullType  on  __Type  {  kind  name  description  fields(includeDeprecated:  true)  {  name  description  args  {  ...InputValue  }  type  {  ...TypeRef  }  isDeprecated  deprecationReason  }  inputFields  {  ...InputValue  }  interfaces  {  ...TypeRef  }  enumValues(includeDeprecated:  true)  {  name  description  isDeprecated  deprecationReason  }  possibleTypes  {  ...TypeRef  }  }  fragment  InputValue  on  __InputValue  {  name  description  type  {  ...TypeRef  }  defaultValue  }  fragment  TypeRef  on  __Type  {  kind  name  ofType  {  kind  name  ofType  {  kind  name  ofType  {  kind  name  ofType  {  kind  name  ofType  {  kind  name  ofType  {  kind  name  ofType  {  kind  name  }  }  }  }  }  }  }  }  
"#;

impl AsyncClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
            snapshot: None,
        }
    }

    pub async fn gql<T>(&self, body: String) -> T
    where
        T: for<'de> serde::de::Deserialize<'de>,
    {
        self.client
            .post(&self.endpoint)
            .body(body)
            .send()
            .await
            .unwrap()
            .json::<T>()
            .await
            .unwrap()
    }

    async fn introspect(&self) -> String {
        self.client
            .post(&self.endpoint)
            .body(json!({"operationName":"IntrospectionQuery", "query": INTROSPECTION_QUERY}).to_string())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    }

    async fn safe_introspect(&self) -> Option<String> {
        if let Ok(response) = self
            .client
            .post(&self.endpoint)
            .body(json!({"operationName":"IntrospectionQuery", "query": INTROSPECTION_QUERY}).to_string())
            .send()
            .await
        {
            if let Ok(text) = response.text().await {
                return Some(text);
            }
        }

        None
    }

    /// # Panics
    ///
    /// panics if the set timeout is reached
    pub async fn poll_endpoint(&self, timeout_secs: u64, interval_millis: u64) {
        let start = SystemTime::now();

        loop {
            if self.client.head(&self.endpoint).send().await.is_ok() {
                break;
            }

            assert!(start.elapsed().unwrap().as_secs() < timeout_secs, "timeout");

            sleep(Duration::from_millis(interval_millis));
        }
    }

    pub async fn snapshot(&mut self) {
        self.snapshot = Some(self.introspect().await);
    }

    pub async fn poll_endpoint_for_changes(&mut self, timeout_secs: u64, interval_millis: u64) {
        let start = SystemTime::now();

        loop {
            // panic if a snapshot was not taken
            let snapshot = self.snapshot.clone().unwrap();

            match self.safe_introspect().await {
                Some(current) => {
                    if snapshot != current {
                        self.snapshot = Some(current);
                        break;
                    }
                }
                None => continue,
            };

            assert!(start.elapsed().unwrap().as_secs() < timeout_secs, "timeout");
            sleep(Duration::from_millis(interval_millis));
        }
    }
}
