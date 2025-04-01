#![allow(static_mut_refs)]

use crate::{
    extension::field_resolver::Subscription,
    types::Configuration,
    wit::{self, Error},
};

use super::extension::AnyExtension;

type InitFn =
    Box<dyn Fn(Vec<(String, wit::Schema)>, Configuration) -> Result<Box<dyn AnyExtension>, crate::types::Error>>;

static mut INIT_FN: Option<InitFn> = None;
static mut EXTENSION: Option<Box<dyn AnyExtension>> = None;
static mut SUBSCRIPTION: Option<Box<dyn Subscription>> = None;

/// Initializes the resolver extension with the provided directives using the closure
/// function created with the `register_extension!` macro.
pub(super) fn init(subgraph_schemas: Vec<(String, wit::Schema)>, config: Configuration) -> Result<(), Error> {
    // Safety: This function is only called from the SDK macro, so we can assume that there is only one caller at a time.
    unsafe {
        let init = INIT_FN.as_ref().expect("Resolver extension not initialized correctly.");
        EXTENSION = Some(init(subgraph_schemas, config)?);
    }

    Ok(())
}

/// This function gets called when the extension is registered in the user code with the `register_extension!` macro.
///
/// This should never be called manually by the user.
#[doc(hidden)]
pub(crate) fn register_extension(f: InitFn) {
    // Safety: This function is only called from the SDK macro, so we can assume that there is only one caller at a time.
    unsafe {
        INIT_FN = Some(f);
    }
}

pub(super) fn extension() -> Result<&'static mut dyn AnyExtension, Error> {
    // Safety: This is hidden, only called by us. Every extension call to an instance happens
    // in a single-threaded environment. Do not call this multiple times from different threads.
    unsafe {
        EXTENSION.as_deref_mut().ok_or_else(|| Error {
            message: "Extension was not initialized correctly.".to_string(),
            extensions: Vec::new(),
        })
    }
}

pub(super) fn set_subscription(subscription: Box<dyn Subscription>) {
    unsafe {
        SUBSCRIPTION = Some(subscription);
    }
}

pub(super) fn subscription() -> Result<&'static mut dyn Subscription, Error> {
    unsafe {
        SUBSCRIPTION.as_deref_mut().ok_or_else(|| Error {
            message: "No active subscription.".to_string(),
            extensions: Vec::new(),
        })
    }
}
