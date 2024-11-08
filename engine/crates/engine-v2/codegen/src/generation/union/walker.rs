use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt};
use tracing::instrument;

use crate::{
    domain::{AccessKind, Definition, Domain, Scalar, Union, UnionKind},
    WALKER_TRAIT,
};

use super::{debug::DebugVariantBranch, VariantContext};

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
    let context_accessor = domain.domain_accessor(None);
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
                            pub(#private) id: #id_struct_name,
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
                code_sections.push(quote! {
                    #allow_unused
                    impl<'a> #walker_name<'a> {
                        #[allow(clippy::should_implement_trait)]
                        pub #public fn as_ref(&self) -> &'a #enum_name {
                            &self.#context_accessor[self.id]
                        }
                        pub #public fn id(&self) -> #id_struct_name {
                            self.id
                        }
                        pub #public fn variant(&self) -> #walker_enum_name<'a> {
                            let #context_name = self.#context_name;
                            match self.as_ref() {
                                #(#walk_branches),*
                            }
                        }
                    }
                });

                code_sections.push(quote! {
                    impl<'a> #walk_trait<#context_type> for #id_struct_name {
                        type Walker<'w> = #walker_name<'w> where 'a: 'w;

                        fn walk<'w>(self, #context_name: #context_type) -> Self::Walker<'w>
                        where
                            Self: 'w,
                            'a: 'w
                        {
                            #walker_name {
                                #context_name,
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
            let walk_branches = variants.iter().copied().map(|variant| IdUnionWalkerBranch {
                variant,
                walker_enum_name: union.walker_enum_name(),
                enum_name: union.enum_name(),
            });

            code_sections.push(quote! {
                impl<'a> #walk_trait<#context_type> for #enum_name {
                    type Walker<'w> = #walker_enum_name<'w> where 'a: 'w;

                    fn walk<'w>(self, #context_name: #context_type) -> Self::Walker<'w>
                    where
                        Self: 'w,
                        'a: 'w
                    {
                        match self {
                            #(#walk_branches),*
                        }
                    }
                }
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
                        impl #walker_enum_name<'_> {
                            pub #public fn id(&self) -> #enum_name {
                                match self {
                                    #(#id_branches),*
                                }
                            }
                        }
                    });
                }
                Err((variant_name, value_name)) => {
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
        let tt = if let Some(value) = self.0.value {
            match value {
                Definition::Scalar(Scalar::Value { copy, .. }) => {
                    let walker = Ident::new(value.walker_name(), Span::call_site());
                    if *copy {
                        quote! { #variant(#walker) }
                    } else {
                        quote! { #variant(&'a #walker) }
                    }
                }
                Definition::Scalar(Scalar::Ref { target, .. }) => {
                    let walker = Ident::new(target.walker_name(), Span::call_site());
                    quote! { #variant(#walker<'a>) }
                }
                _ => {
                    let walker = Ident::new(value.walker_name(), Span::call_site());
                    quote! { #variant(#walker<'a>) }
                }
            }
        } else {
            quote! { #variant }
        };
        tokens.append_all(tt);
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
                    let ctx = self.variant.domain.domain_accessor(value.external_domain_name());
                    quote! { #enum_::#variant(id) => #walker::#variant(&#ctx[id]) }
                }
                AccessKind::IdWalker => {
                    let ctx = self.variant.domain.context_accessor(value.external_domain_name());
                    quote! { #enum_::#variant(id) => #walker::#variant(id.walk(#ctx)) }
                }
                AccessKind::ItemWalker => {
                    let ctx = self.variant.domain.context_accessor(value.external_domain_name());
                    quote! { #enum_::#variant(item) => #walker::#variant(item.walk(#ctx)) }
                }
                AccessKind::RefWalker => {
                    let ctx = self.variant.domain.context_accessor(value.external_domain_name());
                    quote! { #enum_::#variant(ref item) => #walker::#variant(item.walk(#ctx)) }
                }
            }
        } else {
            quote! { #enum_::#variant => #walker::#variant }
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
        let enum_ = Ident::new(self.enum_name, Span::call_site());
        let walker = Ident::new(self.walker_enum_name, Span::call_site());
        let variant = Ident::new(&self.variant.name, Span::call_site());

        let tt = match self.variant.value {
            Some(Definition::Scalar(Scalar::Value {
                external_domain_name, ..
            })) => {
                let ctx = self.variant.domain.domain_accessor(external_domain_name.as_deref());
                quote! {
                    #enum_::#variant(id) => #walker::#variant(&#ctx[id])
                }
            }
            Some(value) => {
                let ctx = self.variant.domain.context_accessor(value.external_domain_name());
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
                    #enum_::#variant => #walker::#variant
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
            Some(ty) => {
                return Err((&self.variant.variant.name, ty.name()));
            }
            None => {
                quote! {
                    #walker::#variant => #enum_::#variant
                }
            }
        };

        Ok(tt)
    }
}
