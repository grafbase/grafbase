use engine_parser::types::OperationType;
use schema::{ObjectId, Schema};

pub(crate) trait SchemaExt {
    fn schema(&self) -> &Schema;

    fn get_root_object_id(&self, operation_type: OperationType) -> ObjectId {
        match operation_type {
            OperationType::Query => self.schema().root_operation_types.query,
            OperationType::Mutation => self
                .schema()
                .root_operation_types
                .mutation
                .expect("Mutation operation type not supported by schema."),
            OperationType::Subscription => self
                .schema()
                .root_operation_types
                .subscription
                .expect("Subscription operation type not supported by schema."),
        }
    }
}

impl SchemaExt for Schema {
    fn schema(&self) -> &Schema {
        self
    }
}
