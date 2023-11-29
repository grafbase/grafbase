use engine_value::ConstValue;
use schema::introspection::{IntrospectionDataSource, IntrospectionQuery, IntrospectionResolver};

use super::{ExecutionContext, Executor, ExecutorError, ExecutorInput, ExecutorOutput};
use crate::response::ResponseObjectRoot;

mod resolver;

pub struct IntrospectionExecutor<'a> {
    root: ResponseObjectRoot,
    data_source: &'a IntrospectionDataSource,
    query: IntrospectionQuery,
}

impl<'a> IntrospectionExecutor<'a> {
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn build<'ctx, 'input>(
        ctx: ExecutionContext<'ctx>,
        resolver: &IntrospectionResolver,
        input: ExecutorInput<'input>,
    ) -> Result<Executor<'a>, ExecutorError>
    where
        'ctx: 'a,
    {
        Ok(Executor::Introspection(IntrospectionExecutor {
            root: input.root_response_objects.root(),
            query: resolver.query,
            data_source: ctx.engine.schema[resolver.data_source_id].as_introspection().unwrap(),
        }))
    }

    #[allow(clippy::panic)]
    pub(super) async fn execute(
        self,
        ctx: ExecutionContext<'_>,
        output: &mut ExecutorOutput,
    ) -> Result<(), ExecutorError> {
        // There is no IO, we directly write into the response.
        let mut data = output.data.lock().await;
        for (response_key, field) in ctx.default_walk_selection_set().collect_fields(self.root.object_id) {
            let mut resolver = resolver::Resolver::new(&ctx.engine.schema, self.data_source, &mut data);
            let value = match self.query {
                IntrospectionQuery::Type => {
                    // There is a single argument if any so don't need to match anything, the
                    // query is already validated.
                    let name = field
                        .bound_arguments()
                        .next()
                        .map(|arg| match arg.resolved_value() {
                            ConstValue::String(s) => s,
                            _ => panic!("Validation failure: Expected string argument"),
                        })
                        .expect("Validation failure: missing argument");
                    resolver.type_by_name(field, &name)
                }
                IntrospectionQuery::Schema => resolver.schema(field),
            };
            data.get_mut(self.root.id).fields.insert(response_key, value);
        }
        Ok(())
    }
}
