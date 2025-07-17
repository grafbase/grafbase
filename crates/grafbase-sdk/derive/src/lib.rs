use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::DeriveInput;

/// A proc macro for generating initialization code for a resolver extension.
#[proc_macro_derive(ResolverExtension)]
pub fn resolver_extension(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    init("resolver", ast)
}

/// A proc macro for generating initialization code for a contracts extension.
#[proc_macro_derive(ContractsExtension)]
pub fn contracts_extension(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    init("contracts", ast)
}

/// A proc macro for generating initialization code for an authentication extension.
#[proc_macro_derive(AuthenticationExtension)]
pub fn authentication_extension(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    init("authentication", ast)
}

/// A proc macro for generating initialization code for an authentication extension.
#[proc_macro_derive(AuthorizationExtension)]
pub fn authorization_extension(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    init("authorization", ast)
}

#[proc_macro_derive(HooksExtension)]
pub fn hooks_extension(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    init("hooks", ast)
}

fn init(ext: &str, ast: DeriveInput) -> TokenStream {
    let ext = Ident::new(ext, Span::call_site());
    let name = &ast.ident;

    let (_, ty_generics, _) = ast.generics.split_for_impl();

    let ts = quote! {
        #[doc(hidden)]
        #[unsafe(export_name = "register-extension")]
        pub extern "C" fn __register_extension() {
            grafbase_sdk::extension::#ext::register::<#name #ty_generics>();
        }
    };

    ts.into()
}
