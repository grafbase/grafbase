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
use event_queue::EventQueue;
use extension_catalog::ExtensionId;
pub use field_resolver::*;
pub use hooks::*;
pub use resolver::*;
pub use selection_set_resolver::*;

pub trait Anything<'a>: serde::Serialize + Send + 'a {}
impl<'a, T> Anything<'a> for T where T: serde::Serialize + Send + 'a {}

pub trait EngineExtensions:
    AuthorizationExtension
    + FieldResolverExtension
    + SelectionSetResolverExtension
    + ResolverExtension
    + ContractsExtension
    + EngineHooksExtension
    + Send
    + Sync
    + 'static
{
}

pub trait GatewayExtensions: GatewayHooksExtension + AuthenticationExtension + Send + Sync + 'static {}

pub trait ExtensionContext: Clone + Default + Send + Sync + 'static {
    fn event_queue(&self) -> &EventQueue;
}

pub trait OnRequestContext: Send + Sync + 'static {
    fn event_queue(&self) -> &EventQueue;
    fn hooks_context(&self) -> &[u8];
}

pub trait AuthenticatedContext: OnRequestContext {
    fn token(&self) -> TokenRef<'_>;
}

pub trait AuthorizedContext: AuthenticatedContext {
    fn authorization_context(&self) -> &[(ExtensionId, Vec<u8>)];
}
