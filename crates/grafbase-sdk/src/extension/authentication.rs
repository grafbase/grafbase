use crate::{
    types::{Configuration, Directive, ErrorResponse, Token},
    wit::Headers,
    Error,
};

use super::Extension;

type InitFn = Box<dyn Fn(Vec<Directive>, Configuration) -> Result<Box<dyn Authenticator>, Box<dyn std::error::Error>>>;

pub(super) static mut EXTENSION: Option<Box<dyn Authenticator>> = None;
pub static mut INIT_FN: Option<InitFn> = None;

pub(super) fn get_extension() -> Result<&'static mut dyn Authenticator, Error> {
    // Safety: This is hidden, only called by us. Every extension call to an instance happens
    // in a single-threaded environment. Do not call this multiple times from different threads.
    unsafe {
        EXTENSION.as_deref_mut().ok_or_else(|| Error {
            message: "Resolver extension not initialized correctly.".to_string(),
            extensions: Vec::new(),
        })
    }
}

/// Initializes the resolver extension with the provided directives using the closure
/// function created with the `register_extension!` macro.
pub(super) fn init(directives: Vec<Directive>, configuration: Configuration) -> Result<(), Box<dyn std::error::Error>> {
    // Safety: This function is only called from the SDK macro, so we can assume that there is only one caller at a time.
    unsafe {
        let init = INIT_FN.as_ref().expect("Resolver extension not initialized correctly.");
        EXTENSION = Some(init(directives, configuration)?);
    }

    Ok(())
}

/// This function gets called when the extension is registered in the user code with the `register_extension!` macro.
///
/// This should never be called manually by the user.
#[doc(hidden)]
pub fn register(f: InitFn) {
    // Safety: This function is only called from the SDK macro, so we can assume that there is only one caller at a time.
    unsafe {
        INIT_FN = Some(f);
    }
}

/// A trait that extends `Extension` and provides authentication functionality.
pub trait Authenticator: Extension {
    fn authenticate(&mut self, headers: Headers) -> Result<Token, ErrorResponse>;
}
