#![allow(static_mut_refs)]

use crate::{
    extension::{authorization::ResponseAuthorizer, resolver::Subscription},
    types::{Configuration, SchemaDirective},
    wit::{Error, QueryElements},
};

use super::extension::AnyExtension;

type InitFn =
    Box<dyn Fn(Vec<SchemaDirective>, Configuration) -> Result<Box<dyn AnyExtension>, Box<dyn std::error::Error>>>;

static mut INIT_FN: Option<InitFn> = None;
static mut EXTENSION: Option<Box<dyn AnyExtension>> = None;
static mut SUBSCRIPTION: Option<Box<dyn Subscription>> = None;
static mut AUTHORIZER_CONTEXT: Option<QueryElements> = None;
static mut RESPONSE_AUTHORIZER: Option<Box<dyn ResponseAuthorizer<'static>>> = None;

/// Initializes the resolver extension with the provided directives using the closure
/// function created with the `register_extension!` macro.
pub(super) fn init(directives: Vec<SchemaDirective>, config: Configuration) -> Result<(), Box<dyn std::error::Error>> {
    // Safety: This function is only called from the SDK macro, so we can assume that there is only one caller at a time.
    unsafe {
        let init = INIT_FN.as_ref().expect("Resolver extension not initialized correctly.");
        EXTENSION = Some(init(directives, config)?);
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

pub(super) fn set_authorizer_context(context: QueryElements) -> &'static QueryElements {
    unsafe {
        AUTHORIZER_CONTEXT = Some(context);
        AUTHORIZER_CONTEXT.as_ref().unwrap()
    }
}

pub(super) fn authorizer_context() -> Result<&'static QueryElements, Error> {
    unsafe {
        AUTHORIZER_CONTEXT.as_ref().ok_or_else(|| Error {
            message: "No active authorizer context.".to_string(),
            extensions: Vec::new(),
        })
    }
}

pub(super) unsafe fn drop_authorizer_context() -> Result<(), Error> {
    unsafe {
        AUTHORIZER_CONTEXT
            .take()
            .ok_or_else(|| Error {
                message: "No active authorizer context.".to_string(),
                extensions: Vec::new(),
            })
            .map(|_| ())
    }
}

pub(super) fn set_response_authorizer(authorizer: Box<dyn ResponseAuthorizer<'static>>) {
    unsafe {
        RESPONSE_AUTHORIZER = Some(authorizer);
    }
}

pub(super) fn take_response_authorizer() -> Option<Box<dyn ResponseAuthorizer<'static>>> {
    unsafe { RESPONSE_AUTHORIZER.take() }
}
