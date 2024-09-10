use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt};

use crate::{
    domain::{Definition, Domain},
    GENERATED_MODULE, WALKER_TRAIT,
};

use super::Imports;

pub(super) fn generate_imports<'a>(
    domain: &'a Domain,
    current_module_path: &[String],
    mut imports: Imports<'a>,
) -> anyhow::Result<TokenStream> {
    let root = {
        let mut ts = quote! { crate:: };
        for module in &domain.root_module {
            let module = Ident::new(module, Span::call_site());
            ts.append_all(quote! { #module:: })
        }
        ts
    };

    let mut scalar_imports = Vec::new();
    let mut generated_imports = Vec::new();
    for name in imports.generated {
        let definition = &domain.definitions_by_name[name];
        let meta = match definition {
            Definition::Object(object) => &object.meta,
            Definition::Union(union) => &union.meta,
            Definition::Scalar(scalar) => {
                let name = Ident::new(definition.storage_type().name(), Span::call_site());
                scalar_imports.push(quote! { ,#name });
                if scalar.is_record {
                    let name = Ident::new(&scalar.name, Span::call_site());
                    scalar_imports.push(quote! { ,#name })
                }
                continue;
            }
        };

        if meta.module_path.starts_with(current_module_path) {
            continue;
        }

        let storage_name = Ident::new(definition.storage_type().name(), Span::call_site());
        let walker_name = Ident::new(definition.walker_name(), Span::call_site());
        if generated_imports.is_empty() {
            generated_imports.push(quote! { #storage_name, #walker_name })
        } else {
            generated_imports.push(quote! { ,#storage_name, #walker_name })
        }
    }
    let generated_imports = if !generated_imports.is_empty() {
        let generated_module_name = Ident::new(GENERATED_MODULE, Span::call_site());
        quote! { ,#generated_module_name::{#(#generated_imports)*} }
    } else {
        quote! {}
    };

    imports.walker_lib.insert(WALKER_TRAIT);
    let walker_lib_imports = imports
        .walker_lib
        .into_iter()
        .map(|name| Ident::new(name, Span::call_site()));

    Ok(quote! {
        use walker::{#(#walker_lib_imports),*};
        use #root{
            prelude::*
            #generated_imports
            #(#scalar_imports)*
        };
    })
}
