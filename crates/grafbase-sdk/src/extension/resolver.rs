use crate::{
    types::{Configuration, Directive, FieldDefinition, FieldInputs, FieldOutput},
    wit::{Error, SharedContext},
};

use super::Extension;

type InitFn = Box<dyn Fn(Vec<Directive>, Configuration) -> Result<Box<dyn Resolver>, Box<dyn std::error::Error>>>;

pub(super) static mut EXTENSION: Option<Box<dyn Resolver>> = None;
pub static mut INIT_FN: Option<InitFn> = None;

pub(super) fn get_extension() -> Result<&'static mut dyn Resolver, Error> {
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

/// A trait that extends `Extension` and provides functionality for resolving fields.
///
/// Implementors of this trait are expected to provide a method to resolve field values based on
/// the given context, directive, and inputs. This is typically used in scenarios where field
/// resolution logic needs to be encapsulated within a resolver object, allowing for modular
/// and reusable code design.
pub trait Resolver: Extension {
    /// Resolves a field using the provided context, directive, and input values.
    ///
    /// # Arguments
    ///
    /// * `context` - A reference to a shared context that contains any necessary state or
    ///   environment information needed for resolution.
    /// * `directive` - The directive associated with the field being resolved. Directives can
    ///   modify how fields are resolved, providing additional instructions or constraints.
    /// * `inputs` - A vector of `FieldInput` values that provide input parameters required
    ///   for resolving the field.
    ///
    /// # Returns
    ///
    /// This method returns a `Result` which is:
    /// * `Ok(FieldOutput)` on successful resolution, containing the resolved output values.
    /// * `Err(Error)` if an error occurs during the resolution process, encapsulating details
    ///   about what went wrong. This will prevent resolving of the field.
    ///
    /// The `FieldOutput` type has multiple response values, which can be either successful or an
    /// error result.
    fn resolve_field(
        &mut self,
        context: SharedContext,
        directive: Directive,
        definition: FieldDefinition,
        inputs: FieldInputs,
    ) -> Result<FieldOutput, Error>;
}
