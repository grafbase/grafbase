mod expand;

use proc_macro::TokenStream;
use syn::{ItemImpl, parse_macro_input};

/// Registers the hook implementations to the gateway. This macro must be added to the
/// local implementation of the `Hooks` trait.
#[proc_macro_attribute]
pub fn grafbase_hooks(_: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemImpl);
    expand::expand(&item)
}
