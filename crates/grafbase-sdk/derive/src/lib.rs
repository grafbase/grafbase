use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(ResolverExtension)]
pub fn resolver_extension(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    expand(resolver_init(ast))
}

#[proc_macro_derive(AuthenticationExtension)]
pub fn authentication_extension(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    expand(authentication_init(ast))
}

fn expand(init: proc_macro2::TokenStream) -> TokenStream {
    let token_stream = quote! {
        #[doc(hidden)]
        #[export_name = "register-extension"]
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
        let init_fn = |directives, config| {
            let result = <#name #ty_generics as grafbase_sdk::Extension>::new(directives, config);
            result.map(|extension| Box::new(extension) as Box<dyn grafbase_sdk::Resolver>)
        };

        grafbase_sdk::extension::resolver::register(Box::new(init_fn));
    }
}

fn authentication_init(ast: DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;

    let (_, ty_generics, _) = ast.generics.split_for_impl();

    quote! {
        let init_fn = |directives, config| {
            let result = <#name #ty_generics as grafbase_sdk::Extension>::new(directives, config);
            result.map(|extension| Box::new(extension) as Box<dyn grafbase_sdk::Authenticator>)
        };

        grafbase_sdk::extension::authentication::register(Box::new(init_fn));
    }
}
