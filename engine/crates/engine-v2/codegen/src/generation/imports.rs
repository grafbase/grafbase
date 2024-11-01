use std::collections::HashMap;

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
    let mut scalar_imports = Vec::new();
    let mut generated_imports = Vec::new();
    let mut other_imports = HashMap::<_, Vec<_>>::new();
    for name in imports.generated {
        let definition = &domain.definitions_by_name[name];
        let imports = if let Some(domain_name) = definition.external_domain_name() {
            other_imports.entry(domain_name).or_default()
        } else if matches!(definition, Definition::Scalar(_)) {
            &mut scalar_imports
        } else {
            &mut generated_imports
        };
        let meta = match definition {
            Definition::Object(object) => &object.meta,
            Definition::Union(union) => &union.meta,
            Definition::Scalar(scalar) => {
                if !scalar.in_prelude {
                    let name = Ident::new(definition.storage_type().name(), Span::call_site());
                    imports.push(quote! { ,#name });
                    if scalar.is_record {
                        let name = Ident::new(&scalar.name, Span::call_site());
                        imports.push(quote! { ,#name })
                    }
                }
                continue;
            }
        };

        if meta.module_path.starts_with(current_module_path) {
            continue;
        }

        let storage_name = Ident::new(definition.storage_type().name(), Span::call_site());
        let walker_name = Ident::new(definition.walker_name(), Span::call_site());
        if imports.is_empty() {
            imports.push(quote! { #storage_name, #walker_name })
        } else {
            imports.push(quote! { ,#storage_name, #walker_name })
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

    let other_imports = other_imports
        .into_iter()
        .map(|(name, imports)| {
            let module = &domain.imported_domains[name].module;
            quote! { use #module::{#(#imports),*}; }
        })
        .fold(quote! {}, |mut ts, import| {
            ts.append_all(import);
            ts
        });

    let domain_module = &domain.module;
    Ok(quote! {
        use walker::{#(#walker_lib_imports),*};
        use #domain_module::{
            prelude::*
            #generated_imports
            #(#scalar_imports)*
        };
        #other_imports
    })
}
