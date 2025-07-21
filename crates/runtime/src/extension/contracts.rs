use std::future::Future;

use engine_schema::Schema;

pub trait ContractsExtension<Context>: Clone + Send + Sync + 'static {
    fn construct(&self, context: &Context, key: String, schema: Schema) -> impl Future<Output = Option<Schema>> + Send;
}
