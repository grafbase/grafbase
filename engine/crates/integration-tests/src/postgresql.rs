use crate::{Engine, EngineBuilder};
use async_once_cell::OnceCell;
use engine::Response;
use futures::FutureExt;
use graphql_parser::parse_schema;
use indoc::formatdoc;
use postgresql_types::transport::{ExecuteResponse, NeonTransport, QueryResponse, Transport};
use serde::de::DeserializeOwned;
use std::{collections::HashMap, future::Future, panic::AssertUnwindSafe, sync::Arc};

static ADMIN_CONNECTION_STRING: &str = "postgres://postgres:grafbase@db.localtest.me:5432/postgres";

#[track_caller]
pub fn query_postgresql<F, U>(test: F) -> String
where
    F: FnOnce(TestApi) -> U,
    U: Future<Output = Response>,
{
    let database = super::random_name();
    let test_api = || async { TestApi::new(&database).await };

    inner_query_postgresql(test_api, &database, test)
}

#[track_caller]
pub fn query_namespaced_postgresql<F, U>(name: &str, test: F) -> String
where
    F: FnOnce(TestApi) -> U,
    U: Future<Output = Response>,
{
    let database = super::random_name();
    let test_api = || async { TestApi::new_namespaced(&database, name).await };

    inner_query_postgresql(test_api, &database, test)
}

#[track_caller]
pub fn introspect_postgresql<F, U>(schema_init: F) -> String
where
    F: FnOnce(TestApi) -> U,
    U: Future<Output = ()>,
{
    let database = super::random_name();
    let test_api = || async { TestApi::new(&database).await };

    inner_introspect_postgresql(test_api, &database, schema_init)
}

#[track_caller]
pub fn introspect_namespaced_postgresql<F, U>(name: &str, schema_init: F) -> String
where
    F: FnOnce(TestApi) -> U,
    U: Future<Output = ()>,
{
    let database = super::random_name();
    let test_api = || async { TestApi::new_namespaced(&database, name).await };

    inner_introspect_postgresql(test_api, &database, schema_init)
}

#[track_caller]
fn inner_introspect_postgresql<B, E, S, T>(api: S, database: &str, schema_init: B) -> String
where
    B: FnOnce(TestApi) -> E,
    E: Future<Output = ()>,
    S: FnOnce() -> T,
    T: Future<Output = TestApi>,
{
    super::runtime().block_on(async {
        let admin = NeonTransport::new("dummy-ray-id", ADMIN_CONNECTION_STRING).unwrap();

        admin
            .execute(&format!("DROP DATABASE IF EXISTS {database}"))
            .await
            .unwrap();

        admin.execute(&format!("CREATE DATABASE {database}")).await.unwrap();

        let api = api().await;
        let response = AssertUnwindSafe(schema_init(api.clone())).catch_unwind().await;

        response.expect("Error in test execution.");

        let builder = EngineBuilder::new(&api.inner.schema);

        let result = parser_sdl::parse(&api.inner.schema, &HashMap::new(), false, &builder)
            .await
            .expect("error in parsing the schema")
            .registry
            .export_sdl(false);

        parse_schema::<String>(&result).unwrap().to_string()
    })
}

#[track_caller]
fn inner_query_postgresql<P, L, U, R>(api: U, database: &str, test: P) -> String
where
    P: FnOnce(TestApi) -> L,
    L: Future<Output = Response>,
    U: FnOnce() -> R,
    R: Future<Output = TestApi>,
{
    super::runtime().block_on(async {
        let admin = NeonTransport::new("dummy-ray-id", ADMIN_CONNECTION_STRING).unwrap();

        admin
            .execute(&format!("DROP DATABASE IF EXISTS {database}"))
            .await
            .unwrap();

        admin.execute(&format!("CREATE DATABASE {database}")).await.unwrap();

        let api = api().await;
        let response = AssertUnwindSafe(test(api.clone())).catch_unwind().await;

        let response = response.expect("Error in test execution.");

        serde_json::to_string_pretty(&response.to_graphql_response()).unwrap()
    })
}

pub struct Inner {
    engine: OnceCell<Engine>,
    connection: NeonTransport,
    schema: String,
}

#[derive(Clone)]
pub struct TestApi {
    inner: Arc<Inner>,
}

impl TestApi {
    async fn new(database: &str) -> Self {
        let mut url = url::Url::parse(ADMIN_CONNECTION_STRING).unwrap();
        url.set_path(&format!("/{database}"));

        let connection_string = url.to_string();

        let schema = formatdoc! {r#"
            extend schema
              @postgresql(
                name: "test",
                url: "{connection_string}",
                namespace: false
              )    
        "#};

        Self::new_inner(schema, connection_string).await
    }

    async fn new_namespaced(database: &str, name: &str) -> Self {
        let mut url = url::Url::parse(ADMIN_CONNECTION_STRING).unwrap();
        url.set_path(&format!("/{database}"));

        let connection_string = url.to_string();

        let schema = formatdoc! {r#"
            extend schema
              @postgresql(
                name: "{name}",
                url: "{connection_string}",
                namespace: true
              )
        "#};

        Self::new_inner(schema, connection_string).await
    }

    async fn new_inner(schema: String, connection_string: String) -> Self {
        let engine = OnceCell::new();
        let connection = NeonTransport::new("dummy-ray-id", &connection_string).unwrap();

        let inner = Inner {
            engine,
            connection,
            schema,
        };

        Self { inner: Arc::new(inner) }
    }

    pub async fn execute_sql(&self, query: &str) -> ExecuteResponse {
        self.inner
            .connection
            .execute(query)
            .await
            .expect("error in query execute")
    }

    pub async fn execute(&self, operation: impl AsRef<str>) -> Response {
        Box::pin(
            self.inner
                .engine
                // this prevents a race. we initialize the engine only when executing the first request,
                // so the introspection runs only after we've modified the database schema.
                .get_or_init(async { Engine::new(self.inner.schema.clone()).await }),
        )
        .await
        .execute(operation.as_ref())
        .await
    }

    pub async fn execute_as<T>(&self, operation: impl AsRef<str>) -> T
    where
        T: DeserializeOwned + Send,
    {
        let result = self.execute(operation).await;
        let response = serde_json::to_string(&result.to_graphql_response()).unwrap();

        println!("{response}");

        serde_json::from_str(&response).unwrap()
    }

    pub async fn query_sql<T>(&self, query: &str) -> QueryResponse<T>
    where
        T: DeserializeOwned + Send,
    {
        self.inner.connection.query(query).await.expect("error in query")
    }
}
