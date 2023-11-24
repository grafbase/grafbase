use futures_util::{stream::BoxStream, Future};
use postgres_connector_types::{database_definition::DatabaseDefinition, error::Error, transport::Transport};
use runtime::pg::{PgTransportFactory, PgTransportFactoryError, PgTransportFactoryInner, PgTransportFactoryResult};
use send_wrapper::SendWrapper;
use std::{collections::HashMap, pin::Pin, sync::Arc};
use wasm_bindgen::JsValue;

pub(crate) struct WasmTransport {
    pub(crate) connection_string: String,
    pub(crate) callbacks: SendWrapper<Arc<super::PgCallbacks>>,
}

impl WasmTransport {
    async fn execute(&self, query: &str, params: Vec<serde_json::Value>) -> Result<u64, JsValue> {
        #[cfg(target_arch = "wasm32")]
        {
            todo!()
            // let context = JsValue::null();
            // let query = JsValue::from_str(query);
            // let params = params.try_into()?;
            // self.callbacks.as_ref().parameterized_execute.call2(context, query, params).await
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(serde_json::Value::Null)
        }
    }

    async fn query(&self, query: &str, params: Vec<serde_json::Value>) -> Result<serde_json::Value, JsValue> {
        #[cfg(target_arch = "wasm32")]
        {
            tracing::info!("querying");
            let context = JsValue::null();
            let query = JsValue::from_str(query);
            tracing::info!("query: {query:?}");
            let params = serde_wasm_bindgen::to_value(&params)?;
            tracing::info!("params: {params:?}");
            let result = self
                .callbacks
                .as_ref()
                .parameterized_query
                .call2(&context, &query, &params)?;
            tracing::info!("result: {result:?}");
            let result = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(result)).await?;
            Ok(serde_wasm_bindgen::from_value(result)?)
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(serde_json::Value::Null)
        }
    }
}

#[cfg(target_arch = "wasm32")]
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[cfg(not(target_arch = "wasm32"))]
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

impl Transport for WasmTransport {
    fn parameterized_query<'a>(
        &'a self,
        query: &'a str,
        params: Vec<serde_json::Value>,
    ) -> BoxStream<'a, Result<serde_json::Value, Error>> {
        Box::pin(futures_util::stream::once(send_wrapper::SendWrapper::new(async move {
            self.query(query, params).await.map_err(|e| Error::Query {
                code: "0".to_owned(),
                message: e.as_string().unwrap_or_else(|| format!("{e:?}")),
            })
        })))
    }

    fn connection_string(&self) -> &str {
        self.connection_string.as_str()
    }

    fn close<'a>(self) -> BoxFuture<'a, postgres_connector_types::Result<()>>
    where
        Self: 'a,
    {
        Box::pin(async move { Ok(()) })
    }

    fn parameterized_execute<'b, 'query, 'a>(
        &'b self,
        query: &'query str,
        params: Vec<serde_json::Value>,
    ) -> BoxFuture<'a, postgres_connector_types::Result<i64>>
    where
        'b: 'a,
        'query: 'a,
        Self: 'a,
    {
        todo!()
    }
}

pub(crate) fn make_pg_transport_factory(transports: HashMap<String, WasmTransport>) -> PgTransportFactory {
    let factory_impl = PgTransportFactoryImpl { transports };
    PgTransportFactory::new(Box::new(factory_impl))
}

struct PgTransportFactoryImpl {
    transports: HashMap<String, WasmTransport>,
}

#[async_trait::async_trait]
impl PgTransportFactoryInner for PgTransportFactoryImpl {
    async fn try_new(
        &self,
        name: &str,
        database_definition: &DatabaseDefinition,
    ) -> PgTransportFactoryResult<Box<dyn Transport>> {
        tracing::error!("got to the factory new");
        panic!()
    }

    fn fetch_cached(&self, name: &str) -> PgTransportFactoryResult<&dyn Transport> {
        tracing::info!("fetching cached transport `{name}`");
        self.transports
            .get(name)
            .map(|t| t as &dyn Transport)
            .ok_or_else(|| PgTransportFactoryError::TransportNotFound(name.to_owned()))
    }
}
