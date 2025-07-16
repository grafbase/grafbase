use std::future::Future;

use engine_schema::Schema;
use error::ErrorResponse;

pub trait ContractsExtension<Context>: Clone + Send + Sync + 'static {
    fn construct(
        &self,
        context: &Context,
        key: String,
        schema: Schema,
    ) -> impl Future<Output = Result<Schema, ErrorResponse>> + Send;
}
