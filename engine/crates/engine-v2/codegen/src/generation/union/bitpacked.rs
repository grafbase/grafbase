use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, TokenStreamExt};
use tracing::instrument;

use crate::domain::{BitPackedIdUnion, StorageType};

use super::VariantContext;

#[instrument(skip_all)]
pub(super) fn generate_bitpacked_id_union(
    union: &BitPackedIdUnion,
    variants: &[VariantContext<'_>],
) -> anyhow::Result<Vec<TokenStream>> {
    let bit_offset = (usize::BITS - variants.len().leading_zeros()) as usize;
    let enum_ = Ident::new(&union.enum_name, Span::call_site());
    let bitpacked_enum_ = Ident::new(&union.bitpacked_enum_name(), Span::call_site());

    let mut code_sections = Vec::new();

    code_sections.push({
        let from_branches = variants.iter().copied().map(|variant| PackBranch {
            enum_name: &union.enum_name,
            variant,
            size: &union.size,
            bit_offset,
        });
        quote! {
            impl From<#enum_> for #bitpacked_enum_ {
                fn from(id: #enum_) -> Self {
                    let value = match id {
                        #(#from_branches),*
                    };
                    #bitpacked_enum_(std::num::NonZero::new(value).unwrap())
                }
            }
        }
    });

    code_sections.push({
        let bit_offset_plus_1 = Literal::usize_unsuffixed(bit_offset + 1);
        let bit_offset = Literal::usize_unsuffixed(bit_offset);
        let into_branches = variants.iter().copied().map(|variant| UnpackBranch {
            variant,
            enum_name: &union.enum_name,
        });
        quote! {
            impl From<#bitpacked_enum_> for #enum_ {
                fn from(id: #bitpacked_enum_) -> Self {
                    let id = id.0.get();
                    let ty = id & ((1 << #bit_offset_plus_1) - 1);
                    let id = id >> #bit_offset;
                    match ty {
                        #(#into_branches),*,
                        _ => unreachable!("Unknown type {ty}"),
                    }
                }
            }
        }
    });

    code_sections.push({
        let size = Ident::new(&union.size, Span::call_site());
        quote! {
            #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
            pub struct #bitpacked_enum_(std::num::NonZero<#size>);
        }

    });

    code_sections.push(quote! {
        impl #bitpacked_enum_ {
            pub fn unpack(self) -> #enum_ {
                self.into()
            }
        }
    });

    code_sections.push(quote! {
        impl #enum_ {
            pub fn pack(self) -> #bitpacked_enum_ {
                self.into()
            }
        }
    });

    Ok(code_sections)
}

struct PackBranch<'a> {
    variant: VariantContext<'a>,
    enum_name: &'a str,
    bit_offset: usize,
    size: &'a str,
}

impl quote::ToTokens for PackBranch<'_> {
    #[instrument(name = "pack", skip_all, fields(variant = ?self.variant.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let enum_ = Ident::new(self.enum_name, Span::call_site());
        let size = Ident::new(self.size, Span::call_site());
        let variant_name = Ident::new(&self.variant.name, Span::call_site());
        let bit_offset = Literal::usize_unsuffixed(self.bit_offset);
        let index = Literal::usize_unsuffixed(self.variant.index + 1);

        tokens.append_all(quote! {
            #enum_::#variant_name(id) => (#size::from(id) << #bit_offset) + #index
        });
    }
}

struct UnpackBranch<'a> {
    variant: VariantContext<'a>,
    enum_name: &'a str,
}

impl quote::ToTokens for UnpackBranch<'_> {
    #[instrument(name = "unpack", skip_all, fields(variant = ?self.variant.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let enum_ = Ident::new(self.enum_name, Span::call_site());
        let variant_index = Literal::usize_unsuffixed(self.variant.index + 1);
        let variant_name = Ident::new(&self.variant.name, Span::call_site());
        if let Some(ty) = self.variant.value {
            let id_struct_name = match ty.storage_type() {
                StorageType::Id { name, .. } => name,
                _ => {
                    tracing::error!("{} has no id", ty.name());
                    unreachable!()
                }
            };
            let id_struct_name = Ident::new(id_struct_name, Span::call_site());
            tokens.append_all(quote! {
                #variant_index => #enum_::#variant_name(#id_struct_name::from(id))
            });
        } else {
            tokens.append_all(quote! {
                #variant_index => #enum_::#variant_name
            });
        }
    }
}
