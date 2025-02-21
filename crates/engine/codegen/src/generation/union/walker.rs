use case::CaseExt;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{TokenStreamExt, quote};
use tracing::instrument;

use crate::{
    WALKER_TRAIT,
    domain::{AccessKind, Definition, Domain, Scalar, Union, UnionKind},
};

use super::{VariantContext, debug::DebugVariantBranch};

#[instrument(skip_all)]
pub fn generate_walker(
    domain: &Domain,
    union: &Union,
    variants: &[VariantContext<'_>],
) -> anyhow::Result<Vec<TokenStream>> {
    let public = &domain.public_visibility;
    let allow_unused = if domain.public_visibility.is_empty() {
        quote! {}
    } else {
        quote! { #[allow(unused)] }
    };
    let enum_name = Ident::new(union.enum_name(), Span::call_site());
    let context_name = Ident::new(&domain.context_name, Span::call_site());
    let context_type = &domain.context_type;
    let walker_enum_name = Ident::new(union.walker_enum_name(), Span::call_site());
    let walk_trait = Ident::new(WALKER_TRAIT, Span::call_site());
    let doc = union
        .description
        .as_ref()
        .map(|desc| {
            let desc = proc_macro2::Literal::string(desc);
            quote! { #[doc = #desc] }
        })
        .unwrap_or_default();
    let private = {
        let m = &domain.module;
        quote! { in #m }
    };

    let mut code_sections = Vec::new();

    let walker_variants = variants.iter().copied().map(WalkerVariant);
    code_sections.push(quote! {
        #doc
        #[derive(Clone, Copy)]
        pub #public enum #walker_enum_name<'a> {
            #(#walker_variants),*
        }
    });

    let debug_variants = variants.iter().copied().map(|variant| DebugVariantBranch {
        variant,
        is_walker: true,
        enum_name: union.walker_enum_name(),
    });
    code_sections.push(quote! {
        impl std::fmt::Debug for #walker_enum_name<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#debug_variants)*
                }
            }
        }
    });

    match &union.kind {
        UnionKind::Record(record) => {
            if let Some(indexed) = &record.indexed {
                let walker_name = Ident::new(union.walker_name(), Span::call_site());
                let id_struct_name = Ident::new(&indexed.id_struct_name, Span::call_site());

                code_sections.insert(
                    0,
                    quote! {
                        #[derive(Clone, Copy)]
                        pub #public struct #walker_name<'a> {
                            pub(#private) #context_name: #context_type,
                            pub #public id: #id_struct_name,
                        }
                    },
                );
                code_sections.push(quote! {
                    impl std::ops::Deref for #walker_name<'_> {
                        type Target = #enum_name;
                        fn deref(&self) -> &Self::Target {
                            self.as_ref()
                        }
                    }
                });

                let walk_branches = variants.iter().copied().map(|variant| RecordUnionWalkerBranch {
                    variant,
                    walker_enum_name: union.walker_enum_name(),
                    enum_name: union.enum_name(),
                });
                let as_variants = variants.iter().copied().map(|variant| AsVariantWalkerVariant {
                    variant,
                    walker_enum_name: union.walker_enum_name(),
                });

                let domain_accesor = domain.domain_accessor();
                code_sections.push(quote! {
                    #allow_unused
                    impl<'a> #walker_name<'a> {
                        #[allow(clippy::should_implement_trait)]
                        pub #public fn as_ref(&self) -> &'a #enum_name {
                            &self.#domain_accesor[self.id]
                        }
                        pub #public fn variant(&self) -> #walker_enum_name<'a> {
                            let #context_name = self.#context_name;
                            match self.as_ref() {
                                #(#walk_branches),*
                            }
                        }
                        #(#as_variants)*
                    }
                });

                code_sections.push(quote! {
                    impl<'a> #walk_trait<#context_type> for #id_struct_name {
                        type Walker<'w> = #walker_name<'w> where 'a: 'w;

                        fn walk<'w>(self, #context_name: impl Into<#context_type>) -> Self::Walker<'w>
                        where
                            Self: 'w,
                            'a: 'w
                        {
                            #walker_name {
                                #context_name: #context_name.into(),
                                id: self,
                            }
                        }
                    }
                });

                code_sections.push(quote! {
                    impl std::fmt::Debug for #walker_name<'_> {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            self.variant().fmt(f)
                        }
                    }
                });
            } else {
                tracing::error!("Could not generate a walker, it's neither an @id nor @indexed.",);
                unimplemented!()
            }
        }
        UnionKind::Id(_) | UnionKind::BitpackedId(_) => {
            let from_variants = variants.iter().copied().map(|variant| FromNonScalarWalkerVariant {
                variant,
                walker_enum_name: union.walker_enum_name(),
            });
            code_sections.push(quote! { #(#from_variants)* });

            let walk_branches = variants.iter().copied().map(|variant| IdUnionWalkerBranch {
                variant,
                walker_enum_name: union.walker_enum_name(),
                enum_name: union.enum_name(),
            });

            code_sections.push(quote! {
                impl<'a> #walk_trait<#context_type> for #enum_name {
                    type Walker<'w> = #walker_enum_name<'w> where 'a: 'w;

                    fn walk<'w>(self, #context_name: impl Into<#context_type>) -> Self::Walker<'w>
                    where
                        Self: 'w,
                        'a: 'w
                    {
                        let #context_name: #context_type = #context_name.into();
                        match self {
                            #(#walk_branches),*
                        }
                    }
                }
            });

            let as_variants = variants.iter().copied().map(|variant| AsIdWalkerVariant {
                variant,
                walker_enum_name: union.walker_enum_name(),
            });

            match variants
                .iter()
                .copied()
                .map(|variant| {
                    IdUnionWalkerIdMethodBranch {
                        variant,
                        walker_enum_name: union.walker_enum_name(),
                        enum_name: union.enum_name(),
                    }
                    .try_to_tokens()
                })
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(id_branches) => {
                    code_sections.push(quote! {
                        #allow_unused
                        impl<'a> #walker_enum_name<'a> {
                            pub #public fn id(&self) -> #enum_name {
                                match self {
                                    #(#id_branches),*
                                }
                            }
                            #(#as_variants)*
                        }
                    });
                }
                Err((variant_name, value_name)) => {
                    code_sections.push(quote! {
                        #allow_unused
                        impl<'a> #walker_enum_name<'a> {
                            #(#as_variants)*
                        }
                    });
                    tracing::warn!(
                        "Could not generate id() method for walker '{}' because variant '{}' has a value '{}' which doesn't have any.",
                        union.name(),
                        variant_name,
                        value_name
                    );
                }
            }
        }
    }

    Ok(code_sections)
}

struct WalkerVariant<'a>(VariantContext<'a>);

impl quote::ToTokens for WalkerVariant<'_> {
    #[instrument(name = "walker_variant", skip_all, fields(variant = ?self.0.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant = Ident::new(&self.0.name, Span::call_site());
        tokens.append_all(if let Some(ty) = self.0.value_type() {
            quote! { #variant(#ty) }
        } else {
            let context_type = &self.0.domain.context_type;
            quote! { #variant(#context_type) }
        })
    }
}

struct RecordUnionWalkerBranch<'a> {
    variant: VariantContext<'a>,
    enum_name: &'a str,
    walker_enum_name: &'a str,
}

impl quote::ToTokens for RecordUnionWalkerBranch<'_> {
    #[instrument(name = "record_union_walker_branch", skip_all, fields(variant = ?self.variant.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ctx = Ident::new(&self.variant.domain.context_name, Span::call_site());
        let enum_ = Ident::new(self.enum_name, Span::call_site());
        let variant = Ident::new(&self.variant.name, Span::call_site());
        let walker = Ident::new(self.walker_enum_name, Span::call_site());

        let tt = if let Some(value) = self.variant.value {
            match value.access_kind() {
                AccessKind::Copy => {
                    quote! { #enum_::#variant(item) => #walker::#variant(item) }
                }
                AccessKind::Ref => {
                    quote! { #enum_::#variant(item) => #walker::#variant(&item) }
                }
                AccessKind::IdRef => {
                    quote! { #enum_::#variant(id) => #walker::#variant(&#ctx[id]) }
                }
                AccessKind::IdWalker => {
                    quote! { #enum_::#variant(id) => #walker::#variant(id.walk(#ctx)) }
                }
                AccessKind::ItemWalker => {
                    quote! { #enum_::#variant(item) => #walker::#variant(item.walk(#ctx)) }
                }
                AccessKind::RefWalker => {
                    quote! { #enum_::#variant(item) => #walker::#variant(item.walk(#ctx)) }
                }
            }
        } else {
            quote! { #enum_::#variant => #walker::#variant(#ctx) }
        };
        tokens.append_all(tt);
    }
}

struct IdUnionWalkerBranch<'a> {
    variant: VariantContext<'a>,
    enum_name: &'a str,
    walker_enum_name: &'a str,
}

impl quote::ToTokens for IdUnionWalkerBranch<'_> {
    #[instrument(name = "id_union_walker_branch", skip_all, fields(variant = ?self.variant.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ctx = Ident::new(&self.variant.domain.context_name, Span::call_site());
        let enum_ = Ident::new(self.enum_name, Span::call_site());
        let walker = Ident::new(self.walker_enum_name, Span::call_site());
        let variant = Ident::new(&self.variant.name, Span::call_site());

        let tt = match self.variant.value {
            Some(Definition::Scalar(Scalar::Value { .. })) => {
                quote! {
                    #enum_::#variant(id) => #walker::#variant(&#ctx[id])
                }
            }
            Some(value) => {
                if value.storage_type().is_id() {
                    quote! {
                        #enum_::#variant(id) => #walker::#variant(id.walk(#ctx))
                    }
                } else {
                    quote! {
                        #enum_::#variant(item) => #walker::#variant(item.walk(#ctx))
                    }
                }
            }
            _ => {
                quote! {
                    #enum_::#variant => #walker::#variant(#ctx)
                }
            }
        };
        tokens.append_all(tt);
    }
}

struct IdUnionWalkerIdMethodBranch<'a> {
    variant: VariantContext<'a>,
    enum_name: &'a str,
    walker_enum_name: &'a str,
}

impl<'a> IdUnionWalkerIdMethodBranch<'a> {
    fn try_to_tokens(&self) -> Result<TokenStream, (&'a str, &'a str)> {
        let enum_ = Ident::new(self.enum_name, Span::call_site());
        let walker = Ident::new(self.walker_enum_name, Span::call_site());
        let variant = Ident::new(&self.variant.name, Span::call_site());

        let tt = match self.variant.value {
            Some(Definition::Scalar(scalar)) => match scalar {
                Scalar::Record { indexed, .. } if indexed.is_some() => quote! {
                    #walker::#variant(walker) => #enum_::#variant(walker.id)
                },

                Scalar::Value { copy, .. } if *copy => quote! {
                    #walker::#variant(item) => #enum_::#variant(item)
                },

                _ => {
                    return Err((&self.variant.variant.name, scalar.name()));
                }
            },
            Some(Definition::Object(object)) => {
                if object.indexed.is_some() {
                    quote! {
                        #walker::#variant(walker) => #enum_::#variant(walker.id)
                    }
                } else if object.copy {
                    quote! {
                        #walker::#variant(walker) => #enum_::#variant(walker.item)
                    }
                } else {
                    return Err((&self.variant.variant.name, &object.name));
                }
            }
            Some(Definition::Union(Union {
                kind: UnionKind::Id(_), ..
            })) => {
                quote! {
                    #walker::#variant(walker) => #enum_::#variant(walker.id())
                }
            }
            Some(ty) => {
                return Err((&self.variant.variant.name, ty.name()));
            }
            None => {
                quote! {
                    #walker::#variant(_) => #enum_::#variant
                }
            }
        };

        Ok(tt)
    }
}

struct FromNonScalarWalkerVariant<'a> {
    variant: VariantContext<'a>,
    walker_enum_name: &'a str,
}

impl quote::ToTokens for FromNonScalarWalkerVariant<'_> {
    #[instrument(name = "from_walker_variant", skip_all, fields(variant = ?self.variant.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let enum_ = Ident::new(self.walker_enum_name, Span::call_site());
        let variant = Ident::new(&self.variant.name, Span::call_site());
        if self
            .variant
            .value
            .map(|def| matches!(def, Definition::Scalar { .. }))
            .unwrap_or_default()
        {
            return;
        }
        let Some(ty) = self.variant.value_type() else {
            return;
        };

        tokens.append_all(quote! {
            impl<'a> From<#ty> for #enum_<'a> {
                fn from(item: #ty) -> Self {
                    #enum_::#variant(item)
                }
            }
        });
    }
}

struct AsVariantWalkerVariant<'a> {
    variant: VariantContext<'a>,
    walker_enum_name: &'a str,
}

impl quote::ToTokens for AsVariantWalkerVariant<'_> {
    #[instrument(name = "as_walker_variant", skip_all, fields(variant = ?self.variant.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let public = &self.variant.domain.public_visibility;
        let enum_ = Ident::new(self.walker_enum_name, Span::call_site());
        let variant = Ident::new(&self.variant.name, Span::call_site());
        let is_variant = Ident::new(&format!("is_{}", self.variant.name.to_snake()), Span::call_site());

        if let Some(ty) = self.variant.value_type() {
            let as_variant = Ident::new(&format!("as_{}", self.variant.name.to_snake()), Span::call_site());
            tokens.append_all(quote! {
                pub #public fn #is_variant(&self) -> bool {
                    matches!(self.variant(), #enum_::#variant(_))
                }
                pub #public fn #as_variant(&self) -> Option<#ty> {
                    match self.variant() {
                        #enum_::#variant(item) => Some(item),
                        _ => None
                    }
                }
            });
        } else {
            tokens.append_all(quote! {
                pub #public fn #is_variant(&self) -> bool {
                    matches!(self.variant(), #enum_::#variant(_))
                }
            });
        }
    }
}

struct AsIdWalkerVariant<'a> {
    variant: VariantContext<'a>,
    walker_enum_name: &'a str,
}

impl quote::ToTokens for AsIdWalkerVariant<'_> {
    #[instrument(name = "as_walker_variant", skip_all, fields(variant = ?self.variant.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let public = &self.variant.domain.public_visibility;
        let enum_ = Ident::new(self.walker_enum_name, Span::call_site());
        let variant = Ident::new(&self.variant.name, Span::call_site());
        let is_variant = Ident::new(&format!("is_{}", self.variant.name.to_snake()), Span::call_site());
        let as_variant = Ident::new(&format!("as_{}", self.variant.name.to_snake()), Span::call_site());
        let Some(ty) = self.variant.value_type() else {
            return;
        };

        tokens.append_all(quote! {
            pub #public fn #is_variant(&self) -> bool {
                matches!(self, #enum_::#variant(_))
            }
            pub #public fn #as_variant(&self) -> Option<#ty> {
                match self {
                    #enum_::#variant(item) => Some(*item),
                    _ => None
                }
            }
        });
    }
}
