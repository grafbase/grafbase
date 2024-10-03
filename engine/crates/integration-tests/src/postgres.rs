use std::{collections::HashMap, future::Future, panic::AssertUnwindSafe, sync::Arc};

use engine::{registry::RegistrySdlExt, Response};
use futures::FutureExt;
use graphql_parser::parse_schema;
use indoc::formatdoc;
use postgres_connector_types::transport::{PooledTcpTransport, Transport, TransportExt};
use runtime_local::LazyPgConnectionsPool;
use serde::de::DeserializeOwned;
use tokio::sync::OnceCell;

use crate::{Engine, EngineBuilder};

pub async fn admin_pool() -> &'static PooledTcpTransport {
    // this is for creating/dropping databases, which _should not be done_ over pgbouncer.
    const ADMIN_CONNECTION_STRING: &str = "postgres://postgres:grafbase@localhost:5432/postgres";

    static POOL: OnceCell<PooledTcpTransport> = OnceCell::const_new();
    POOL.get_or_init(|| async {
        PooledTcpTransport::new(
            ADMIN_CONNECTION_STRING,
            postgres_connector_types::transport::PoolingConfig {
                max_size: Some(32),
                ..Default::default()
            },
        )
        .await
        .unwrap()
    })
    .await
}

// url for the engine for introspecting, querying and mutating the database.
const BASE_CONNECTION_STRING: &str = "postgres://postgres:grafbase@localhost:5432/";

#[track_caller]
pub fn query_postgres<F, U>(test: F) -> String
where
    F: FnOnce(TestApi) -> U,
    U: Future<Output = Response>,
{
    let database = super::random_name();
    let test_api = || async { TestApi::new(&database).await };

    inner_query_postgres(test_api, &database, test)
}

#[track_caller]
pub fn query_namespaced_postgres<F, U>(name: &str, test: F) -> String
where
    F: FnOnce(TestApi) -> U,
    U: Future<Output = Response>,
{
    let database = super::random_name();
    let test_api = || async { TestApi::new_namespaced(&database, name).await };

    inner_query_postgres(test_api, &database, test)
}

#[track_caller]
pub fn introspect_postgres<F, U>(schema_init: F) -> String
where
    F: FnOnce(TestApi) -> U,
    U: Future<Output = ()>,
{
    let database = super::random_name();
    let test_api = || async { TestApi::new(&database).await };

    inner_introspect_postgres(test_api, &database, schema_init)
}

#[track_caller]
pub fn introspect_namespaced_postgres<F, U>(name: &str, schema_init: F) -> String
where
    F: FnOnce(TestApi) -> U,
    U: Future<Output = ()>,
{
    let database = super::random_name();
    let test_api = || async { TestApi::new_namespaced(&database, name).await };

    inner_introspect_postgres(test_api, &database, schema_init)
}

#[track_caller]
fn inner_introspect_postgres<B, E, S, T>(api: S, database: &str, schema_init: B) -> String
where
    B: FnOnce(TestApi) -> E,
    E: Future<Output = ()>,
    S: FnOnce() -> T,
    T: Future<Output = TestApi>,
{
    super::runtime().block_on(async {
        let admin = admin_pool().await;
        admin
            .execute(&format!("DROP DATABASE IF EXISTS {database}"))
            .await
            .unwrap();

        admin.execute(&format!("CREATE DATABASE {database}")).await.unwrap();

        let api = api().await;
        let response = AssertUnwindSafe(schema_init(api.clone())).catch_unwind().await;

        response.expect("Error in test execution.");

        let builder = EngineBuilder::new(&api.inner.schema);

        let result = parser_sdl::parse(&api.inner.schema, &HashMap::new(), &builder)
            .await
            .expect("error in parsing the schema")
            .registry
            .export_sdl(false);

        parse_schema::<String>(&result).unwrap().to_string()
    })
}

#[track_caller]
fn inner_query_postgres<P, L, U, R>(api: U, database: &str, test: P) -> String
where
    P: FnOnce(TestApi) -> L,
    L: Future<Output = Response>,
    U: FnOnce() -> R,
    R: Future<Output = TestApi>,
{
    super::runtime().block_on(async {
        let admin = admin_pool().await;

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
    connection_string: String,
    connection_pool: LazyPgConnectionsPool,
    schema: String,
}

#[derive(Clone)]
pub struct TestApi {
    inner: Arc<Inner>,
}

impl TestApi {
    async fn connection(&self) -> Arc<dyn Transport> {
        self.inner.connection_pool.get(&self.inner.connection_string).await
    }

    async fn engine(&self) -> &Engine {
        self.inner
            .engine
            // this prevents a race. we initialize the engine only when executing the first request,
            // so the introspection runs only after we've modified the database schema.
            .get_or_init(|| async {
                EngineBuilder::new_with_pool(self.inner.schema.clone(), self.inner.connection_pool.clone())
                    .build()
                    .await
            })
            .await
    }

    async fn new(database: &str) -> Self {
        let mut url = url::Url::parse(BASE_CONNECTION_STRING).unwrap();
        url.set_path(&format!("/{database}"));

        let connection_string = url.to_string();

        let schema = formatdoc! {r#"
            extend schema
              @postgres(
                name: "test",
                url: "{connection_string}",
                namespace: false
              )
        "#};

        Self::new_inner(schema, connection_string).await
    }

    async fn new_namespaced(database: &str, name: &str) -> Self {
        let mut url = url::Url::parse(BASE_CONNECTION_STRING).unwrap();
        url.set_path(&format!("/{database}"));

        let connection_string = url.to_string();

        let schema = formatdoc! {r#"
            extend schema
              @postgres(
                name: "{name}",
                url: "{connection_string}",
                namespace: true
              )
        "#};

        Self::new_inner(schema, connection_string).await
    }

    async fn new_inner(schema: String, connection_string: String) -> Self {
        let engine = OnceCell::new();

        let inner = Inner {
            engine,
            connection_pool: LazyPgConnectionsPool::new(move |connection_string| async move {
                PooledTcpTransport::new(
                    &connection_string,
                    postgres_connector_types::transport::PoolingConfig {
                        max_size: Some(1),
                        wait_timeout: None,
                        create_timeout: None,
                        recycle_timeout: None,
                    },
                )
                .await
                .unwrap()
            }),
            connection_string,
            schema,
        };

        Self { inner: Arc::new(inner) }
    }

    pub async fn execute_sql(&self, query: &str) -> i64 {
        self.connection()
            .await
            .execute(query)
            .await
            .expect("error in query execute")
    }

    pub async fn execute(&self, operation: impl AsRef<str>) -> Response {
        self.engine().await.execute(operation.as_ref()).await
    }

    pub async fn execute_parameterized(
        &self,
        operation: impl AsRef<str>,
        variables: impl serde::Serialize,
    ) -> Response {
        self.engine()
            .await
            .execute(operation.as_ref())
            .variables(variables)
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

    pub async fn query_sql<T>(&self, query: &str) -> Vec<T>
    where
        T: DeserializeOwned + Send,
    {
        self.connection()
            .await
            .collect_query(query, Vec::new())
            .await
            .expect("error in query")
    }

    pub async fn row_count(&self, table: &str) -> usize {
        #[derive(serde::Deserialize)]
        struct Result {
            count: String,
        }

        let query = format!("SELECT COUNT(*) AS count FROM \"{table}\"");
        let response = self.query_sql::<Result>(&query).await;

        response.into_iter().next().unwrap().count.parse().unwrap()
    }
}
