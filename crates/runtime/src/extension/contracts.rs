use std::future::Future;

use engine_schema::Schema;

pub trait ContractsExtension: Clone + Send + Sync + 'static {
    fn construct(&self, key: String, schema: Schema) -> impl Future<Output = Option<Schema>> + Send;
}
