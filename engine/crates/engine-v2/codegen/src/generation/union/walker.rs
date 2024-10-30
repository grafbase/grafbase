use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt};
use tracing::instrument;

use crate::{
    domain::{AccessKind, Definition, Domain, Union, UnionKind},
    WALKER_TRAIT,
};

use super::{debug::DebugVariantBranch, VariantContext};

#[instrument(skip_all)]
pub fn generate_walker(
    domain: &Domain,
    union: &Union,
    variants: &[VariantContext<'_>],
) -> anyhow::Result<Vec<TokenStream>> {
    let enum_name = Ident::new(union.enum_name(), Span::call_site());
    let graph_name = Ident::new(&domain.graph_var_name, Span::call_site());
    let graph_type = Ident::new(&domain.graph_type_name, Span::call_site());
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

    let mut code_sections = Vec::new();

    let walker_variants = variants.iter().copied().map(WalkerVariant);
    code_sections.push(quote! {
        #doc
        #[derive(Clone, Copy)]
        pub enum #walker_enum_name<'a> {
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
                        pub struct #walker_name<'a> {
                            pub(crate) #graph_name: &'a #graph_type,
                            pub(crate) id: #id_struct_name,
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
                    impl<'a> #walker_name<'a> {
                        #[allow(clippy::should_implement_trait)]
                        pub fn as_ref(&self) -> &'a #enum_name {
                            &self.#graph_name[self.id]
                        }
                        pub fn id(&self) -> #id_struct_name {
                            self.id
                        }
                        pub fn variant(&self) -> #walker_enum_name<'a> {
                            let #graph_name = self.#graph_name;
                            match self.as_ref() {
                                #(#walk_branches),*
                            }
                        }
                    }
                });

                code_sections.push(quote! {
                    impl #walk_trait<#graph_type> for #id_struct_name {
                        type Walker<'a> = #walker_name<'a>;

                        fn walk<'a>(self, #graph_name: &'a #graph_type) -> Self::Walker<'a>
                        where
                            Self: 'a
                        {
                            #walker_name {
                                #graph_name,
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
                impl #walk_trait<#graph_type> for #enum_name {
                    type Walker<'a> = #walker_enum_name<'a>;

                    fn walk<'a>(self, #graph_name: &'a #graph_type) -> Self::Walker<'a>
                    where
                        Self: 'a
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
                        impl #walker_enum_name<'_> {
                            pub fn id(&self) -> #enum_name {
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
            let walker = Ident::new(value.walker_name(), Span::call_site());
            match value {
                Definition::Scalar(scalar) if !scalar.is_record => {
                    if scalar.copy {
                        quote! { #variant(#walker) }
                    } else {
                        quote! { #variant(&'a #walker) }
                    }
                }
                _ => {
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
            let graph = Ident::new(&self.variant.domain.graph_var_name, Span::call_site());
            match value.access_kind() {
                AccessKind::Copy => {
                    quote! { #enum_::#variant(item) => #walker::#variant(item) }
                }
                AccessKind::Ref => {
                    quote! { #enum_::#variant(item) => #walker::#variant(&item) }
                }
                AccessKind::IdRef => {
                    quote! { #enum_::#variant(id) => #walker::#variant(&#graph[id]) }
                }
                AccessKind::IdWalker => {
                    quote! { #enum_::#variant(id) => #walker::#variant(id.walk(#graph)) }
                }
                AccessKind::ItemWalker => {
                    quote! { #enum_::#variant(item) => #walker::#variant(item.walk(#graph)) }
                }
                AccessKind::RefWalker => {
                    quote! { #enum_::#variant(ref item) => #walker::#variant(item.walk(#graph)) }
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
        let graph = Ident::new(&self.variant.domain.graph_var_name, Span::call_site());

        let tt = match self.variant.value {
            Some(Definition::Scalar(scalar)) if !scalar.is_record => {
                quote! {
                    #enum_::#variant(id) => #walker::#variant(&#graph[id])
                }
            }
            Some(value) => {
                if value.storage_type().is_id() {
                    quote! {
                        #enum_::#variant(id) => #walker::#variant(id.walk(#graph))
                    }
                } else {
                    quote! {
                        #enum_::#variant(item) => #walker::#variant(item.walk(#graph))
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
            Some(Definition::Scalar(scalar)) => {
                if scalar.is_record && scalar.indexed.is_some() {
                    quote! {
                        #walker::#variant(walker) => #enum_::#variant(walker.id)
                    }
                } else if scalar.copy {
                    quote! {
                        #walker::#variant(item) => #enum_::#variant(item)
                    }
                } else {
                    return Err((&self.variant.variant.name, &scalar.name));
                }
            }
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
