use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// A proc macro for generating initialization code for a resolver extension.
///
/// Add it on top of the type which implements `Extension` and `Resolver` traits to
/// register it as a resolver extension.
#[proc_macro_derive(ResolverExtension)]
pub fn resolver_extension(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    expand(resolver_init(ast))
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

fn resolver_init(ast: DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;

    let (_, ty_generics, _) = ast.generics.split_for_impl();

    quote! {
        grafbase_sdk::extension::resolver::register::<#name #ty_generics>();
    }
}

fn authentication_init(ast: DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;

    let (_, ty_generics, _) = ast.generics.split_for_impl();

    quote! {
        grafbase_sdk::extension::authentication::register::<#name #ty_generics>();
    }
}
