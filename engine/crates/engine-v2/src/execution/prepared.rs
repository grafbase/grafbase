use std::{
    future::IntoFuture,
    hash::{Hash, Hasher},
};

use futures_util::future::BoxFuture;
use schema::CacheConfig;

use super::ExecutionCoordinator;
use crate::Response;

pub enum PreparedExecution {
    BadRequest(BadRequest),
    PreparedRequest(PreparedRequest),
}

impl PreparedExecution {
    pub(crate) fn request(coordinator: ExecutionCoordinator) -> Self {
        Self::PreparedRequest(PreparedRequest { coordinator })
    }

    pub(crate) fn bad_request(response: Response) -> Self {
        Self::BadRequest(BadRequest { response })
    }
}

pub struct BadRequest {
    pub(crate) response: Response,
}

pub struct PreparedRequest {
    pub(crate) coordinator: ExecutionCoordinator,
}

impl PreparedRequest {
    pub fn computed_cache_config(&self) -> Option<&CacheConfig> {
        self.coordinator.operation().cache_config.as_ref()
    }

    pub fn operation_hash<H: Hasher>(&self, state: &mut H) {
        self.coordinator.operation_plan_cache_key().hash::<H>(state);
        state.write_usize(self.coordinator.variables().len());
        for (name, variable) in self.coordinator.variables().iter() {
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
            PreparedExecution::PreparedRequest(PreparedRequest { coordinator }) => Box::pin(coordinator.execute()),
        }
    }
}
