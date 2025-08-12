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
    AuthorizationExtension<Self::Context>
    + FieldResolverExtension
    + SelectionSetResolverExtension
    + ResolverExtension<Self::Context>
    + ContractsExtension<Self::Context>
    + EngineHooksExtension<Self::Context>
    + Send
    + Sync
    + 'static
{
    type Context: ExtensionContext;
}

pub trait GatewayExtensions:
    GatewayHooksExtension<Self::Context> + AuthenticationExtension<Self::Context> + Send + Sync + 'static
{
    type Context: ExtensionContext;
}

pub trait ExtensionContext: Clone + Default + Send + Sync + 'static {
    fn event_queue(&self) -> &EventQueue;
}

trait HasHooksContext {
    fn hooks_context(&self) -> &[u8];
}

trait HasToken {
    fn token(&self) -> TokenRef<'_>;
}

trait HasAuthorizationContext {
    fn authorization_context(&self) -> &[(ExtensionId, Vec<u8>)];
}
