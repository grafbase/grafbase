use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(ResolverExtension)]
pub fn resolver_extension(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    expand(resolver_init(ast))
}

fn expand(init: proc_macro2::TokenStream) -> TokenStream {
    let token_stream = quote! {
        #[doc(hidden)]
        #[export_name = "register_extension"]
        pub extern "C" fn __register_extension(host_version: u64) -> i64 {
            let version_result = grafbase_sdk::check_host_version(host_version);

            if version_result < 0 {
                return version_result;
            }

            #init

            version_result
        }
    };

    TokenStream::from(token_stream)
}

fn resolver_init(ast: DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;

    let (_, ty_generics, _) = ast.generics.split_for_impl();

    quote! {
        let init_fn = |directives| {
            let result = <#name #ty_generics as grafbase_sdk::Extension>::new(directives);
            result.map(|extension| Box::new(extension) as Box<dyn grafbase_sdk::Resolver>)
        };

        grafbase_sdk::extension::resolver::register(Box::new(init_fn));
    }
}
