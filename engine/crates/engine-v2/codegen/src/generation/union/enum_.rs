use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt};
use tracing::instrument;

use crate::{
    domain::{Domain, Union, UnionKind},
    generation::docstr,
};

use super::{debug::DebugVariantBranch, VariantContext};

#[instrument(skip_all)]
pub fn generate_enum(
    domain: &Domain,
    union: &Union,
    variants: &[VariantContext<'_>],
) -> anyhow::Result<Vec<TokenStream>> {
    let enum_name = Ident::new(union.enum_name(), Span::call_site());

    let additional_derives = {
        let mut derives = TokenStream::new();
        if !union.meta.derive.is_empty() {
            let names = union.meta.derive.iter().map(|name| Ident::new(name, Span::call_site()));
            derives = quote! { ,#(#names),* };
        }
        match &union.kind {
            UnionKind::Record(record) if record.copy => derives.extend(quote! { , Clone, Copy }),
            UnionKind::Id(_) | UnionKind::BitpackedId(_) => {
                derives.extend(quote! { , Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash })
            }
            _ => {}
        }
        derives
    };

    let docstr = proc_macro2::Literal::string(&docstr::generated_from(domain, union.span));
    let enum_variants = variants.iter().copied().map(EnumVariant);
    let union_enum = quote! {
        #[doc = #docstr]
        #[derive(serde::Serialize, serde::Deserialize #additional_derives)]
        pub enum #enum_name {
            #(#enum_variants),*
        }
    };

    let mut code_sections = vec![union_enum];

    let debug_variants = variants.iter().copied().map(|variant| DebugVariantBranch {
        variant,
        enum_name: union.enum_name(),
    });
    code_sections.push(quote! {
        impl std::fmt::Debug for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#debug_variants)*
                }
            }
        }
    });

    if let UnionKind::Id(union) = &union.kind {
        let from_variants = variants.iter().copied().map(|variant| FromVariant {
            variant,
            enum_name: &union.enum_name,
        });
        code_sections.push(quote! { #(#from_variants)* });
    }

    Ok(code_sections)
}

struct EnumVariant<'a>(VariantContext<'a>);

impl quote::ToTokens for EnumVariant<'_> {
    #[instrument(name = "enum_variant", skip_all, fields(variant = ?self.0.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant = Ident::new(&self.0.name, Span::call_site());
        let tt = if let Some(value) = self.0.value {
            let storage_type = Ident::new(&value.storage_type().to_string(), Span::call_site());
            quote! { #variant(#storage_type) }
        } else {
            quote! { #variant }
        };
        tokens.append_all(tt);
    }
}

struct FromVariant<'a> {
    variant: VariantContext<'a>,
    enum_name: &'a str,
}

impl quote::ToTokens for FromVariant<'_> {
    #[instrument(name = "from_variant", skip_all, fields(variant = ?self.variant.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let enum_ = Ident::new(self.enum_name, Span::call_site());
        let variant = Ident::new(&self.variant.name, Span::call_site());
        let Some(value) = self.variant.value else {
            return;
        };
        let storage_type = Ident::new(&value.storage_type().to_string(), Span::call_site());
        tokens.append_all(quote! {
            impl From<#storage_type> for #enum_ {
                fn from(value: #storage_type) -> Self {
                    #enum_::#variant(value)
                }
            }
        });
    }
}
