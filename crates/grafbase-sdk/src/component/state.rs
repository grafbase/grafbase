#![allow(static_mut_refs)]

use std::{borrow::Cow, ops::DerefMut};

use crate::{
    extension::resolver::{Subscription, SubscriptionCallback},
    host_io::logger::HostLogger,
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
    host_log_level: String,
) -> Result<(), Error> {
    let mut builder = env_filter::Builder::new();

    let host_log_level = parse_host_level(host_log_level);
    builder.parse(&host_log_level);

    let filter = builder.build();
    let logger = HostLogger { filter };

    log::set_boxed_logger(Box::new(logger)).expect("Failed to set logger");
    log::set_max_level(log::LevelFilter::Trace);

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

/// Parses and processes host log level configuration string.
///
/// This function processes a comma-separated string of log level directives and handles
/// extension-specific logging configuration. It performs the following transformations:
///
/// - Extracts log levels from `extension=level` directives
/// - When extension directives are present, filters out standalone log level tokens
///   (`trace`, `debug`, `info`, `warn`, `error`)
/// - When no extension directives are present, preserves all parts unchanged
///
/// # Arguments
///
/// * `host_log_level` - A comma-separated string containing log level directives
///
/// # Returns
///
/// A processed string with the appropriate log level configuration
///
/// # Examples
///
/// ```ignore
/// // With extension directive
/// parse_host_level("extension=debug,info".to_string()) // Returns "debug"
///
/// // Without extension directive
/// parse_host_level("debug,my_module=info".to_string()) // Returns "debug,my_module=info"
/// ```
fn parse_host_level(host_log_level: String) -> String {
    let parts: Vec<&str> = host_log_level.split(',').map(|part| part.trim()).collect();
    let has_extension_directives = parts.iter().any(|part| part.starts_with("extension="));

    parts
        .into_iter()
        .filter_map(|part| {
            // Handle extension=level -> level
            if let Some(level) = part.strip_prefix("extension=") {
                return Some(Cow::Owned(level.to_string()));
            }

            // If extension directives are present, filter out standalone log levels
            if has_extension_directives {
                match part {
                    "trace" | "debug" | "info" | "warn" | "error" => None,
                    _ => Some(Cow::Borrowed(part)),
                }
            } else {
                // Keep all other parts unchanged when no extension directives
                Some(Cow::Borrowed(part))
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_host_level_with_extension_directive() {
        // Single extension directive
        assert_eq!(parse_host_level("extension=debug".to_string()), "debug");
        assert_eq!(parse_host_level("extension=info".to_string()), "info");
        assert_eq!(parse_host_level("extension=trace".to_string()), "trace");
        assert_eq!(parse_host_level("extension=warn".to_string()), "warn");
        assert_eq!(parse_host_level("extension=error".to_string()), "error");
    }

    #[test]
    fn test_parse_host_level_with_extension_and_standalone_levels() {
        // Extension directive with standalone log levels - standalone levels should be filtered out
        assert_eq!(parse_host_level("extension=debug,info".to_string()), "debug");
        assert_eq!(parse_host_level("info,extension=debug".to_string()), "debug");
        assert_eq!(parse_host_level("trace,extension=warn,error".to_string()), "warn");
        assert_eq!(
            parse_host_level("debug,info,extension=error,warn,trace".to_string()),
            "error"
        );
    }

    #[test]
    fn test_parse_host_level_with_extension_and_module_directives() {
        // Extension directive with module-specific directives - module directives should be kept
        assert_eq!(
            parse_host_level("extension=debug,my_module=info".to_string()),
            "debug,my_module=info"
        );
        assert_eq!(
            parse_host_level("my_module=info,extension=debug".to_string()),
            "my_module=info,debug"
        );
        assert_eq!(
            parse_host_level("extension=warn,crate1=debug,crate2=info".to_string()),
            "warn,crate1=debug,crate2=info"
        );
    }

    #[test]
    fn test_parse_host_level_without_extension_directive() {
        // No extension directive - everything should be preserved
        assert_eq!(parse_host_level("debug".to_string()), "debug");
        assert_eq!(parse_host_level("info,warn".to_string()), "info,warn");
        assert_eq!(
            parse_host_level("debug,my_module=info".to_string()),
            "debug,my_module=info"
        );
        assert_eq!(
            parse_host_level("trace,crate1=debug,crate2=info,error".to_string()),
            "trace,crate1=debug,crate2=info,error"
        );
    }

    #[test]
    fn test_parse_host_level_with_whitespace() {
        // Test with various whitespace configurations
        assert_eq!(parse_host_level("extension=debug, info".to_string()), "debug");
        assert_eq!(parse_host_level(" extension=debug , info ".to_string()), "debug");
        assert_eq!(
            parse_host_level("extension=debug,  my_module=info".to_string()),
            "debug,my_module=info"
        );
        assert_eq!(
            parse_host_level(" debug , my_module=info ".to_string()),
            "debug,my_module=info"
        );
    }

    #[test]
    fn test_parse_host_level_edge_cases() {
        // Empty string
        assert_eq!(parse_host_level("".to_string()), "");

        // Only commas - empty parts are preserved
        assert_eq!(parse_host_level(",,,".to_string()), ",,,");

        // Multiple extension directives
        assert_eq!(
            parse_host_level("extension=debug,extension=info".to_string()),
            "debug,info"
        );

        // Extension with empty value
        assert_eq!(parse_host_level("extension=".to_string()), "");

        // Unusual but valid module names
        assert_eq!(
            parse_host_level("extension=debug,my-module=info,my::module=warn".to_string()),
            "debug,my-module=info,my::module=warn"
        );
    }

    #[test]
    fn test_parse_host_level_preserves_order() {
        // Verify that non-filtered items maintain their relative order
        assert_eq!(
            parse_host_level("a=1,extension=debug,b=2,c=3".to_string()),
            "a=1,debug,b=2,c=3"
        );
        assert_eq!(
            parse_host_level("x=info,y=warn,extension=error,z=trace".to_string()),
            "x=info,y=warn,error,z=trace"
        );
    }
}
