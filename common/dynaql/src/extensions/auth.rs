use std::sync::Arc;

use crate::extensions::{Extension, ExtensionContext, ExtensionFactory, NextResolve, ResolveInfo};
use crate::{ServerError, ServerResult, Value};

use logworker::warn;

/// Authorization extension
///
/// This extension will check that the user is authorized to execute the GraphQL operation.
pub struct Auth {
    trace_id: String,
}

impl Auth {
    pub fn new(trace_id: String) -> Self {
        Self { trace_id }
    }
}

impl ExtensionFactory for Auth {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(AuthExtension {
            trace_id: self.trace_id.clone(),
        })
    }
}

struct AuthExtension {
    trace_id: String,
}

#[async_trait::async_trait]
impl Extension for AuthExtension {
    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        if let (Some(required_op), Ok(allowed_ops)) =
            (info.required_operation, ctx.data::<crate::Operations>())
        {
            if !allowed_ops.contains(&required_op) {
                let msg = format!(
                    "Unauthorized to call {name} (missing `{required_op}` operation)",
                    name = info.name
                );
                warn!(self.trace_id, "{msg}");
                return Err(ServerError::new(msg, None));
            }
        }

        next.run(ctx, info).await
    }
}
