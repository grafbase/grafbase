use schema::introspection::{IntrospectionDataSource, IntrospectionResolver};

use super::{Executor, ExecutorError, ExecutorInput};
use crate::{
    execution::ExecutionContext,
    response::{ResponseObjectRoot, ResponsePartBuilder},
};

mod writer;

pub struct IntrospectionExecutor<'a> {
    root: ResponseObjectRoot,
    data_source: &'a IntrospectionDataSource,
}

impl<'a> IntrospectionExecutor<'a> {
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn build<'ctx, 'input>(
        ctx: ExecutionContext<'ctx, 'ctx>,
        resolver: &IntrospectionResolver,
        input: ExecutorInput<'input>,
    ) -> Result<Executor<'a>, ExecutorError>
    where
        'ctx: 'a,
    {
        Ok(Executor::Introspection(IntrospectionExecutor {
            root: input.root_response_objects.root(),
            data_source: ctx.engine.schema[resolver.data_source_id].as_introspection().unwrap(),
        }))
    }

    #[allow(clippy::panic)]
    pub(super) async fn execute(
        self,
        ctx: ExecutionContext<'_, '_>,
        output: &mut ResponsePartBuilder,
    ) -> Result<(), ExecutorError> {
        let introspection_writer = writer::IntrospectionWriter {
            schema: &ctx.engine.schema,
            types: self.data_source,
        };
        ctx.writer(output, self.root)
            .update_with(|writer| match writer.expected_field.name() {
                "__type" => introspection_writer.write_type_field(writer),
                "__schema" => introspection_writer.write_schema_field(writer),
                name => writer::unknown(name),
            });

        Ok(())
    }
}
