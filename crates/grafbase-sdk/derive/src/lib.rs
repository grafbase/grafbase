use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// A proc macro for generating initialization code for a resolver extension.
///
/// Add it on top of the type which implements `Extension` and `Resolver` traits to
/// register it as a resolver extension.
#[proc_macro_derive(FieldResolverExtension)]
pub fn field_resolver_extension(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    expand(field_resolver_init(ast))
}

/// A proc macro for generating initialization code for a selection set resolver extension.
///
/// Add it on top of the type which implements `Extension` and `Resolver` traits to
/// register it as a resolver extension.
#[proc_macro_derive(SelectionSetResolverExtension)]
pub fn selection_set_resolver_extension(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    expand(selection_set_resolver_init(ast))
}

/// A proc macro for generating initialization code for an authentication extension.
///
/// Add it on top of the type which implements `Extension` and `Authenticator` traits to
/// register it as a resolver extension.
#[proc_macro_derive(AuthenticationExtension)]
pub fn authentication_extension(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    expand(authentication_init(ast))
}

/// A proc macro for generating initialization code for an authentication extension.
///
/// Add it on top of the type which implements `Extension` and `Authenticator` traits to
/// register it as a resolver extension.
#[proc_macro_derive(AuthorizationExtension)]
pub fn authorization_extension(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    expand(authorization_init(ast))
}

fn expand(init: proc_macro2::TokenStream) -> TokenStream {
    let token_stream = quote! {
        #[doc(hidden)]
        #[unsafe(export_name = "register-extension")]
        pub extern "C" fn __register_extension() {
            #init
        }
    };

    TokenStream::from(token_stream)
}

fn selection_set_resolver_init(ast: DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;

    let (_, ty_generics, _) = ast.generics.split_for_impl();

    quote! {
        grafbase_sdk::extension::selection_set_resolver::register::<#name #ty_generics>();
    }
}

fn field_resolver_init(ast: DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;

    let (_, ty_generics, _) = ast.generics.split_for_impl();

    quote! {
        grafbase_sdk::extension::field_resolver::register::<#name #ty_generics>();
    }
}

fn authentication_init(ast: DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;

    let (_, ty_generics, _) = ast.generics.split_for_impl();

    quote! {
        grafbase_sdk::extension::authentication::register::<#name #ty_generics>();
    }
}

fn authorization_init(ast: DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;

    let (_, ty_generics, _) = ast.generics.split_for_impl();

    quote! {
        grafbase_sdk::extension::authorization::register::<#name #ty_generics>();
    }
}
