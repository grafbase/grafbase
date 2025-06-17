#![allow(static_mut_refs)]

use std::ops::DerefMut;

use crate::{
    extension::resolver::{Subscription, SubscriptionCallback},
    types::Configuration,
    wit::{self, Error},
};

use super::extension::AnyExtension;

type InitFn =
    Box<dyn FnOnce(Vec<(String, wit::Schema)>, Configuration) -> Result<Box<dyn AnyExtension>, crate::types::Error>>;

static mut INIT_FN: Option<InitFn> = None;
static mut EXTENSION: Option<Box<dyn AnyExtension>> = None;
static mut SUBSCRIPTION: Option<SubscriptionState> = None;
static mut CONTEXT: Option<wit::SharedContext> = None;
static mut CAN_SKIP_SENDING_EVENTS: bool = false;

enum SubscriptionState {
    Uninitialized {
        prepared: Vec<u8>,
        callback: SubscriptionCallback<'static>,
    },
    Initialized(Box<dyn Subscription>),
}

/// Initializes the resolver extension with the provided directives using the closure
/// function created with the `register_extension!` macro.
pub(super) fn init(
    subgraph_schemas: Vec<(String, wit::Schema)>,
    config: Configuration,
    can_skip_sending_events: bool,
) -> Result<(), Error> {
    // Safety: This function is only called from the SDK macro, so we can assume that there is only one caller at a time.
    unsafe {
        let init = std::mem::take(&mut INIT_FN).expect("Resolver extension not initialized correctly.");
        EXTENSION = Some(init(subgraph_schemas, config)?);
        CAN_SKIP_SENDING_EVENTS = can_skip_sending_events;
    }

    Ok(())
}

pub(crate) fn with_context<F, T>(context: wit::SharedContext, f: F) -> T
where
    F: FnOnce() -> T,
{
    // Safety: This function is only called from extension functions by us.
    unsafe {
        CONTEXT = Some(context);
    }

    // Safety: if this panics, the whole extension will be poisoned.
    let res = f();

    // Safety: This function is only called from extension functions by us.
    unsafe {
        CONTEXT = None;
    }

    res
}

#[allow(unused)]
pub(crate) fn current_context() -> &'static wit::SharedContext {
    // SAFETY: We are in a single-threaded environment, this function is internal.
    unsafe { CONTEXT.as_ref().expect("Context not initialized") }
}

// Coarse grained event filtering. Arbitrary logic can be
// used to select only some events on the host side afterwards.
pub(crate) fn can_skip_sending_events() -> bool {
    unsafe { CAN_SKIP_SENDING_EVENTS }
}

pub(crate) fn queue_event(name: &str, data: &[u8]) {
    // SAFETY: This is mutated only by us before extension is called.
    if let Some(ctx) = unsafe { CONTEXT.as_ref() } {
        ctx.push_event(name, data);
    }
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

pub(super) fn set_subscription_callback(prepared: Vec<u8>, callback: SubscriptionCallback<'static>) {
    unsafe {
        SUBSCRIPTION = Some(SubscriptionState::Uninitialized { prepared, callback });
    }
}

pub(super) fn subscription() -> Result<&'static mut dyn Subscription, Error> {
    // Safety: This is hidden, only called by us. Every extension call to an instance happens
    // in a single-threaded environment. Do not call this multiple times from different threads.
    unsafe {
        let state = std::mem::take(&mut SUBSCRIPTION);
        match state {
            Some(SubscriptionState::Initialized(_)) => {
                SUBSCRIPTION = state; // Restore the state
            }
            Some(SubscriptionState::Uninitialized { prepared, callback }) => {
                SUBSCRIPTION = Some(SubscriptionState::Initialized(callback()?));
                // Must be dropped *after* callback as callback may keep a reference to it
                drop(prepared);
            }
            None => {
                return Err(Error {
                    message: "No active subscription.".to_string(),
                    extensions: Vec::new(),
                });
            }
        }
        let Some(SubscriptionState::Initialized(subscription)) = SUBSCRIPTION.as_mut() else {
            unreachable!();
        };
        Ok(subscription.deref_mut())
    }
}

pub(super) fn drop_subscription() {
    // Safety: This is hidden, only called by us. Every extension call to an instance happens
    // in a single-threaded environment. Do not call this multiple times from different threads.
    unsafe {
        SUBSCRIPTION = None;
    }
}
