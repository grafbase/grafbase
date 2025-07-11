mod authentication;
mod authorization;
mod field_resolver;
mod hooks;
mod resolver;
mod selection_set_resolver;

use crate::Error;

pub(crate) use authentication::*;
pub(crate) use authorization::*;
pub(crate) use field_resolver::*;
pub(crate) use hooks::*;
pub(crate) use resolver::*;
pub(crate) use selection_set_resolver::*;

pub trait ExtensionInstance:
    AuthenticationExtensionInstance
    + AuthorizationExtensionInstance
    + FieldResolverExtensionInstance
    + SelectionSetResolverExtensionInstance
    + HooksInstance
    + ResolverExtensionInstance
    + Send
    + 'static
{
    fn recycle(&mut self) -> Result<(), Error>;
}
