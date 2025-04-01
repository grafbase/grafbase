mod authentication;
mod authorization;
mod field_resolver;
mod selection_set_resolver;

use crate::Error;

pub(crate) use authentication::*;
pub(crate) use authorization::*;
pub(crate) use field_resolver::*;
pub(crate) use selection_set_resolver::*;

pub trait ExtensionInstance:
    AuthenticationExtensionInstance
    + AuthorizationExtensionInstance
    + FieldResolverExtensionInstance
    + SelectionSetResolverExtensionInstance
    + Send
    + 'static
{
    fn recycle(&mut self) -> Result<(), Error>;
}
