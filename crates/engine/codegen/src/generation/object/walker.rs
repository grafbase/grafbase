use cynic_parser::common::WrappingType;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use tracing::instrument;

use crate::{
    domain::{AccessKind, Domain, Object},
    WALKER_TRAIT,
};

use super::{debug::WalkerDebug, FieldContext};

#[instrument(skip_all)]
pub fn generate_walker(
    domain: &Domain,
    object: &Object,
    fields: &[FieldContext<'_>],
) -> anyhow::Result<Vec<TokenStream>> {
    let public = &domain.public_visibility;
    let allow_unused = if domain.public_visibility.is_empty() {
        quote! {}
    } else {
        quote! { #[allow(unused)] }
    };
    let context_name = Ident::new(&domain.context_name, Span::call_site());
    let context_type = &domain.context_type;
    let struct_name = Ident::new(&object.struct_name, Span::call_site());
    let walker_name = Ident::new(object.walker_name(), Span::call_site());
    let walk_trait = Ident::new(WALKER_TRAIT, Span::call_site());
    let doc = object
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

    let walker_field_methods = fields.iter().filter(|field| field.has_walker).map(WalkerFieldMethod);
    let mut code_sections = Vec::new();

    if let Some(indexed) = &object.indexed {
        let id_struct_name = Ident::new(&indexed.id_struct_name, Span::call_site());
        let domain_accesor = domain.domain_accessor();

        code_sections.push(quote! {
            #doc
            #[derive(Clone, Copy)]
            pub #public struct #walker_name<'a> {
                pub(#private) #context_name: #context_type,
                pub #public id: #id_struct_name,
            }
        });
        code_sections.push(quote! {
            impl std::ops::Deref for #walker_name<'_> {
                type Target = #struct_name;
                fn deref(&self) -> &Self::Target {
                    self.as_ref()
                }
            }
        });
        code_sections.push(quote! {
            #allow_unused
            impl <'a> #walker_name<'a> {
                #[doc = "Prefer using Deref unless you need the 'a lifetime."]
                #[allow(clippy::should_implement_trait)]
                pub #public fn as_ref(&self) -> &'a #struct_name {
                    &self.#domain_accesor[self.id]
                }
                #(#walker_field_methods)*
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
                        id: self
                    }
                }
            }
        });
    } else if object.copy {
        code_sections.push(quote! {
            #doc
            #[derive(Clone, Copy)]
            pub #public struct #walker_name<'a> {
                pub(#private) #context_name: #context_type,
                pub(#private) item: #struct_name,
            }
        });
        code_sections.push(quote! {
            impl std::ops::Deref for #walker_name<'_> {
                type Target = #struct_name;
                fn deref(&self) -> &Self::Target {
                    &self.item
                }
            }
        });
        code_sections.push(quote! {
            #allow_unused
            impl <'a> #walker_name<'a> {
                #[allow(clippy::should_implement_trait)]
                pub #public fn as_ref(&self) -> &#struct_name {
                    &self.item
                }
                #(#walker_field_methods)*
            }
        });
        code_sections.push(quote! {
            #allow_unused
            impl<'a> #walk_trait<#context_type> for #struct_name {
                type Walker<'w> = #walker_name<'w> where 'a: 'w;

                fn walk<'w>(self, #context_name: impl Into<#context_type>) -> Self::Walker<'w>
                where
                    Self: 'w,
                    'a: 'w
                {
                    #walker_name {
                        #context_name: #context_name.into(),
                        item: self
                    }
                }
            }
        });
    } else {
        code_sections.push(quote! {
            #doc
            #[derive(Clone, Copy)]
            pub #public struct #walker_name<'a> {
                pub(#private) #context_name: #context_type,
                pub(#private) ref_: &'a #struct_name,
            }
        });
        code_sections.push(quote! {
            impl std::ops::Deref for #walker_name<'_> {
                type Target = #struct_name;
                fn deref(&self) -> &Self::Target {
                    self.ref_
                }
            }
        });
        code_sections.push(quote! {
            #allow_unused
            impl <'a> #walker_name<'a> {
                #[allow(clippy::should_implement_trait)]
                pub #public fn as_ref(&self) -> &'a #struct_name {
                    self.ref_
                }
                #(#walker_field_methods)*
            }
        });
        code_sections.push(quote! {
            impl<'a> #walk_trait<#context_type> for &#struct_name {
                type Walker<'w> = #walker_name<'w>
                where
                    Self: 'w,
                    'a: 'w;

                fn walk<'w>(self, #context_name: impl Into<#context_type>) -> Self::Walker<'w>
                where
                    Self: 'w,
                    'a: 'w
                {
                    #walker_name {
                        #context_name: #context_name.into(),
                        ref_: self,
                    }
                }
            }
        });
    }

    if object.meta.debug {
        code_sections.push(WalkerDebug { object, fields }.to_token_stream());
    }

    Ok(code_sections)
}

pub struct WalkerFieldMethod<'a>(&'a FieldContext<'a>);

impl quote::ToTokens for WalkerFieldMethod<'_> {
    #[instrument(name = "walker_field_method", skip_all, fields(field = ?self.0.field))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ctx = Ident::new(&self.0.domain.context_name, Span::call_site());
        let field = Ident::new(&self.0.record_field_name, Span::call_site());
        let method = Ident::new(&self.0.walker_method_name(), Span::call_site());
        let ty = Ident::new(self.0.ty.walker_name(), Span::call_site());
        let kind = self.0.ty.access_kind();

        let return_type_and_body = match self.0.wrapping[..] {
            [] => match kind {
                AccessKind::Copy => quote! {
                    Option<#ty> {
                        self.#field
                    }
                },
                AccessKind::Ref => quote! {
                    Option<&'a #ty> {
                        self.as_ref().#field.as_ref()
                    }
                },
                AccessKind::IdRef => quote! {
                    Option<&'a #ty> {
                        self.#field.walk(self.#ctx)
                    }
                },
                AccessKind::IdWalker => quote! {
                    Option<#ty<'a>> {
                        self.#field.walk(self.#ctx)
                    }
                },
                AccessKind::ItemWalker => quote! {
                    Option<#ty<'a>> {
                        self.as_ref().#field.walk(self.#ctx)
                    }
                },
                AccessKind::RefWalker => quote! {
                    Option<#ty<'a>> {
                        self.as_ref().#field.walk(self.#ctx)
                    }
                },
            },
            [WrappingType::NonNull] => match kind {
                AccessKind::Copy => quote! {
                    #ty {
                        self.#field
                    }
                },
                AccessKind::Ref => quote! {
                    &'a #ty {
                        &self.as_ref().#field
                    }
                },
                AccessKind::IdRef => quote! {
                    &'a #ty {
                        self.#field.walk(self.#ctx)
                    }
                },
                AccessKind::IdWalker => quote! {
                    #ty<'a> {
                        self.#field.walk(self.#ctx)
                    }
                },
                AccessKind::ItemWalker => quote! {
                    #ty<'a> {
                        self.#field.walk(self.#ctx)
                    }
                },
                AccessKind::RefWalker => quote! {
                    #ty<'a> {
                        self.as_ref().#field.walk(self.#ctx)
                    }
                },
            },
            [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull] => match kind {
                AccessKind::Copy => quote! {
                    impl Iter<Item = #ty> + 'a {
                        self.as_ref().#field.iter().copied()
                    }
                },
                AccessKind::Ref | AccessKind::IdRef => quote! {
                    impl Iter<Item = &'a #ty> + 'a {
                        self.as_ref().#field.iter()
                    }
                },
                AccessKind::IdWalker | AccessKind::ItemWalker | AccessKind::RefWalker => {
                    quote! {
                        impl Iter<Item =  #ty<'a>> + 'a {
                            self.as_ref().#field.walk(self.#ctx)
                        }
                    }
                }
            },
            [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull, WrappingType::List, WrappingType::NonNull] => {
                match kind {
                    AccessKind::IdRef => quote! {
                        impl Iter<Item: Iter<Item = &'a #ty> + 'a> + 'a {
                            self.as_ref().#field.walk(self.#ctx)
                        }
                    },
                    accessor => {
                        tracing::error!("Unsupported {accessor:?} for {}", self.0.ty.name());
                        unimplemented!()
                    }
                }
            }
            _ => {
                tracing::error!("Unsupported wrapping {:?}", self.0.wrapping);
                unimplemented!()
            }
        };

        let doc = self
            .0
            .field
            .description
            .as_ref()
            .map(|desc| {
                let desc = proc_macro2::Literal::string(desc);
                quote! { #[doc = #desc] }
            })
            .unwrap_or_default();

        let public = &self.0.domain.public_visibility;
        tokens.append_all(quote! {
            #doc
            pub #public fn #method(&self) -> #return_type_and_body
        });
    }
}
