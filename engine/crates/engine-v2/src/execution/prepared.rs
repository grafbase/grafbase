use std::{
    future::IntoFuture,
    hash::{Hash, Hasher},
    sync::Arc,
};

use engine::RequestHeaders;
use futures_util::future::BoxFuture;
use schema::CacheConfig;

use crate::{plan::OperationPlan, Engine, Response};

use super::{ExecutorCoordinator, Variables};

pub enum PreparedExecution {
    BadRequest(BadRequest),
    PreparedRequest(PreparedRequest),
}

impl PreparedExecution {
    pub(crate) fn bad_request(response: Response) -> Self {
        Self::BadRequest(BadRequest { response })
    }
}

pub struct BadRequest {
    pub(crate) response: Response,
}

pub struct PreparedRequest {
    pub(crate) operation: Arc<OperationPlan>,
    pub(crate) variables: Variables,
    // Keeping the original query for a simpler hash.
    pub(crate) query: String,
    pub(crate) headers: RequestHeaders,
    pub(crate) engine: Arc<Engine>,
}

impl PreparedRequest {
    pub fn computed_cache_config(&self) -> Option<&CacheConfig> {
        self.operation.cache_config.as_ref()
    }

    pub fn operation_hash<H: Hasher>(&self, state: &mut H) {
        self.query.hash::<H>(state);
        self.operation.name.hash(state);
        state.write_usize(self.variables.len());
        for (name, variable) in self.variables.iter() {
            name.hash(state);
            if let Some(value) = &variable.value {
                value.hash::<H>(state);
            } else {
                state.write_u8(0);
            }
        }
    }
}

impl IntoFuture for PreparedExecution {
    type Output = Response;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        match self {
            PreparedExecution::BadRequest(BadRequest { response }) => Box::pin(async move { response }),
            PreparedExecution::PreparedRequest(PreparedRequest {
                operation,
                headers,
                variables,
                engine,
                ..
            }) => Box::pin(async move {
                ExecutorCoordinator::new(engine.as_ref(), operation, variables, headers)
                    .execute()
                    .await
            }),
        }
    }
}
