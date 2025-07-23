mod authentication;
mod authorization;
mod contracts;
mod field_resolver;
mod hooks;
mod resolver;
mod selection_set_resolver;

use crate::InstanceState;

pub(crate) use authentication::*;
pub(crate) use authorization::*;
pub(crate) use contracts::*;
pub(crate) use field_resolver::*;
pub(crate) use hooks::*;
pub(crate) use resolver::*;
pub(crate) use selection_set_resolver::*;
use wasmtime::Store;

pub trait ExtensionInstance:
    AuthenticationExtensionInstance
    + AuthorizationExtensionInstance
    + FieldResolverExtensionInstance
    + SelectionSetResolverExtensionInstance
    + HooksExtensionInstance
    + ResolverExtensionInstance
    + ContractsExtensionInstance
    + Send
    + 'static
{
    fn store(&self) -> &Store<InstanceState>;
}
