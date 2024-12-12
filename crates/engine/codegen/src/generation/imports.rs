use std::collections::HashMap;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt};

use crate::{
    domain::{Definition, Domain, Object, Scalar, Union},
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

    for name in imports.generated.iter().copied() {
        let definition = &domain.definitions_by_name[name];
        let exernal_imports = definition
            .external_domain_name()
            .map(|name| other_imports.entry(name).or_default());
        match definition {
            Definition::Object(Object { meta, .. }) | Definition::Union(Union { meta, .. }) => {
                if exernal_imports.is_none() && meta.module_path.starts_with(current_module_path) {
                    continue;
                }

                let storage_name = Ident::new(definition.storage_type().name(), Span::call_site());
                let walker_name = Ident::new(definition.walker_name(), Span::call_site());
                let tokens = exernal_imports.unwrap_or(&mut generated_imports);
                tokens.push(quote! { #storage_name });
                tokens.push(quote! { #walker_name });
            }
            Definition::Scalar(scalar) => match scalar {
                Scalar::Value { in_prelude, .. } => {
                    if !in_prelude {
                        let name = Ident::new(definition.storage_type().name(), Span::call_site());
                        let tokens = exernal_imports.unwrap_or(&mut scalar_imports);
                        tokens.push(quote! { #name });
                    }
                }
                Scalar::Record { in_prelude, .. } => {
                    if !in_prelude {
                        let name = Ident::new(definition.storage_type().name(), Span::call_site());
                        let walker_name = Ident::new(definition.walker_name(), Span::call_site());
                        let tokens = exernal_imports.unwrap_or(&mut scalar_imports);
                        tokens.push(quote! { #name });
                        tokens.push(quote! { #walker_name });
                    }
                }
                Scalar::Id { name, in_prelude, .. } => {
                    if !in_prelude {
                        let name = Ident::new(name, Span::call_site());
                        let tokens = exernal_imports.unwrap_or(&mut scalar_imports);
                        tokens.push(quote! { #name });
                    }
                }
                Scalar::Ref { in_prelude, target, .. } => {
                    if !in_prelude {
                        let name = Ident::new(definition.storage_type().name(), Span::call_site());
                        let tokens = exernal_imports.unwrap_or(&mut scalar_imports);
                        tokens.push(quote! { #name });
                    }
                    if !imports.generated.contains(&target.name()) && !imports.local.contains(&target.name()) {
                        let tokens = target
                            .external_domain_name()
                            .map(|name| other_imports.entry(name).or_default())
                            .unwrap_or_else(|| match &domain.definitions_by_name[target.name()] {
                                Definition::Scalar(_) => &mut scalar_imports,
                                _ => &mut generated_imports,
                            });
                        let walker_name = Ident::new(target.walker_name(), Span::call_site());
                        tokens.push(quote! { #walker_name });
                    }
                }
            },
        }
    }

    imports.walker_lib.insert(WALKER_TRAIT);
    let walker_lib_imports = imports
        .walker_lib
        .into_iter()
        .map(|name| Ident::new(name, Span::call_site()));

    let mut domain_imports = vec![quote! { prelude::* }];
    if !generated_imports.is_empty() {
        let generated_module_name = Ident::new(GENERATED_MODULE, Span::call_site());
        domain_imports.push(quote! { #generated_module_name::{#(#generated_imports),*} })
    };
    if !scalar_imports.is_empty() {
        domain_imports.push(quote! { #(#scalar_imports),* });
    }

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
        use #domain_module::{#(#domain_imports),*};
        #other_imports
    })
}
