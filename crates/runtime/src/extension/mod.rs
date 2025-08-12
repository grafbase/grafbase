mod authentication;
mod authorization;
mod contracts;
mod field_resolver;
mod hooks;
mod resolver;
mod selection_set_resolver;

pub use authentication::*;
pub use authorization::*;
pub use contracts::*;
pub use field_resolver::*;
pub use hooks::*;
pub use resolver::*;
pub use selection_set_resolver::*;

pub trait Anything<'a>: serde::Serialize + Send + 'a {}
impl<'a, T> Anything<'a> for T where T: serde::Serialize + Send + 'a {}

pub trait EngineExtensions<RequestContext, OperationContext>:
    AuthorizationExtension<RequestContext, OperationContext>
    + FieldResolverExtension
    + SelectionSetResolverExtension
    + ResolverExtension<OperationContext>
    + ContractsExtension
    + EngineHooksExtension<OperationContext>
    + Send
    + Sync
    + 'static
{
}

pub trait GatewayExtensions: GatewayHooksExtension + AuthenticationExtension + Send + Sync + 'static {}
