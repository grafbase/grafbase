use auth::Authorizer;
use engine::{Request, RequestHeaders};
use engine_v2::{CacheableResponse, Engine, EngineRuntime, ExecutionMetadata, Response, Schema};
use futures_util::Stream;

pub struct Gateway {
    engine: Engine,
    authorizer: Box<dyn Authorizer>,
}

impl Gateway {
    pub fn new(schema: Schema, runtime: EngineRuntime, kv: runtime::kv::KvStore) -> Self {
        let authorizer = auth::build(schema.auth_config.as_ref(), &kv);
        let engine = Engine::new(schema, runtime);
        Self { engine, authorizer }
    }

    pub async fn execute<F, E>(
        &self,
        request: Request,
        headers: RequestHeaders,
        serializer: F,
    ) -> Result<CacheableResponse, E>
    where
        F: FnOnce(&Response) -> Result<Vec<u8>, E>,
    {
        if self.authorizer.get_access_token(&headers).await.is_some() {
            let response = self.engine.execute(request, headers).await;
            response.into_cacheable(serializer)
        } else {
            Ok(CacheableResponse {
                bytes: serializer(&Response::error("Unauthorized"))?.into(),
                metadata: ExecutionMetadata::default(),
                has_errors: true,
            })
        }
    }

    pub fn execute_stream(
        &self,
        request: engine::Request,
        headers: RequestHeaders,
    ) -> impl Stream<Item = Response> + '_ {
        self.engine.execute_stream(request, headers)
    }
}
