use std::future::IntoFuture;

use futures_util::future::BoxFuture;
use tracing::{Instrument, Span};

#[cfg(feature = "tracing")]
use grafbase_tracing::span::{GqlRecorderSpanExt, GqlResponseAttributes};

use crate::{request::OperationCacheControl, Response};

use super::ExecutionCoordinator;

pub enum PreparedExecution {
    BadRequest(BadRequest),
    PreparedRequest(PreparedOperation),
}

impl PreparedExecution {
    pub(crate) fn request(coordinator: ExecutionCoordinator, gql_span: Span) -> Self {
        Self::PreparedRequest(PreparedOperation { coordinator, gql_span })
    }

    pub(crate) fn bad_request(response: Response) -> Self {
        Self::BadRequest(BadRequest { response })
    }
}

pub struct BadRequest {
    response: Response,
}

pub struct PreparedOperation {
    coordinator: ExecutionCoordinator,
    gql_span: Span,
}

impl PreparedOperation {
    pub fn cache_control(&self) -> Option<&OperationCacheControl> {
        self.coordinator.operation().cache_control.as_ref()
    }
}

impl IntoFuture for PreparedExecution {
    type Output = Response;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        match self {
            PreparedExecution::BadRequest(BadRequest { response }) => {
                #[cfg(feature = "tracing")]
                Span::current().record_gql_response(GqlResponseAttributes {
                    has_errors: response.has_errors(),
                });

                Box::pin(async move { response })
            }
            PreparedExecution::PreparedRequest(PreparedOperation { coordinator, gql_span }) => {
                Box::pin(coordinator.execute().instrument(gql_span))
            }
        }
    }
}
