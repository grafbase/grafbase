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
    let graph_name = Ident::new(&domain.graph_var_name, Span::call_site());
    let graph_type = Ident::new(&domain.graph_type_name, Span::call_site());
    let struct_name = Ident::new(&object.struct_name, Span::call_site());
    let walker_name = Ident::new(object.walker_name(), Span::call_site());
    let walk_trait = Ident::new(WALKER_TRAIT, Span::call_site());

    let walker_field_methods = fields.iter().filter(|field| field.has_walker).map(WalkerFieldMethod);
    let mut code_sections = Vec::new();

    if let Some(indexed) = &object.indexed {
        let id_struct_name = Ident::new(&indexed.id_struct_name, Span::call_site());

        code_sections.push(quote! {
            #[derive(Clone, Copy)]
            pub struct #walker_name<'a> {
                pub(crate) #graph_name: &'a #graph_type,
                pub(crate) id: #id_struct_name,
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
            impl <'a> #walker_name<'a> {
                #[doc = "Prefer using Deref unless you need the 'a lifetime."]
                #[allow(clippy::should_implement_trait)]
                pub fn as_ref(&self) -> &'a #struct_name {
                    &self.#graph_name[self.id]
                }
                pub fn id(&self) -> #id_struct_name {
                    self.id
                }
                #(#walker_field_methods)*
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
    } else if object.copy {
        code_sections.push(quote! {
            #[derive(Clone, Copy)]
            pub struct #walker_name<'a> {
                pub(crate) #graph_name: &'a #graph_type,
                pub(crate) item: #struct_name,
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
            impl <'a> #walker_name<'a> {
                #[allow(clippy::should_implement_trait)]
                pub fn as_ref(&self) -> &#struct_name {
                    &self.item
                }
                #(#walker_field_methods)*
            }
        });
        code_sections.push(quote! {
            impl #walk_trait<#graph_type> for #struct_name {
                type Walker<'a> = #walker_name<'a>;

                fn walk<'a>(self, #graph_name: &'a #graph_type) -> Self::Walker<'a>
                where
                    Self: 'a
                {
                    #walker_name {
                        #graph_name,
                        item: self,
                    }
                }
            }
        });
    } else {
        code_sections.push(quote! {
            #[derive(Clone, Copy)]
            pub struct #walker_name<'a> {
                pub(crate) #graph_name: &'a #graph_type,
                pub(crate) ref_: &'a #struct_name,
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
            impl <'a> #walker_name<'a> {
                #[allow(clippy::should_implement_trait)]
                pub fn as_ref(&self) -> &'a #struct_name {
                    self.ref_
                }
                #(#walker_field_methods)*
            }
        });
        code_sections.push(quote! {
            impl #walk_trait<#graph_type> for &#struct_name {
                type Walker<'a> = #walker_name<'a>
                where
                    Self: 'a;

                fn walk<'a>(self, #graph_name: &'a #graph_type) -> Self::Walker<'a>
                where
                    Self: 'a
                {
                    #walker_name {
                        #graph_name,
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
        let graph = Ident::new(&self.0.domain.graph_var_name, Span::call_site());
        let field = Ident::new(&self.0.record_field_name, Span::call_site());
        let method = Ident::new(&self.0.walker_method_name(), Span::call_site());
        let ty = Ident::new(self.0.ty.walker_name(), Span::call_site());
        let kind = self.0.ty.access_kind();

        let return_type_and_body = match self.0.wrapping[..] {
            [] => match kind {
                AccessKind::Copy => quote! {
                    Option<#ty> {
                        self.as_ref().#field
                    }
                },
                AccessKind::Ref => quote! {
                    Option<&'a #ty> {
                        self.as_ref().#field.as_ref()
                    }
                },
                AccessKind::IdRef if self.0.ty.name() != self.0.ty.walker_name() => quote! {
                    Option<&'a #ty> {
                        self.as_ref().#field.map(|id| self.#graph[id].as_ref())
                    }
                },
                AccessKind::IdRef => quote! {
                    Option<&'a #ty> {
                        self.as_ref().#field.map(|id| &self.#graph[id])
                    }
                },
                AccessKind::IdWalker => quote! {
                    Option<#ty<'a>> {
                        self.as_ref().#field.as_ref().walk(self.#graph)
                    }
                },
                AccessKind::ItemWalker => quote! {
                    Option<#ty<'a>> {
                        self.as_ref().#field.as_ref().walk(self.#graph)
                    }
                },
                AccessKind::RefWalker => quote! {
                    Option<#ty<'a>> {
                        self.as_ref().#field.as_ref().walk(self.#graph)
                    }
                },
            },
            [WrappingType::NonNull] => match kind {
                AccessKind::Copy => quote! {
                    #ty {
                        self.as_ref().#field
                    }
                },
                AccessKind::Ref => quote! {
                    &'a #ty {
                        &self.as_ref().#field
                    }
                },
                AccessKind::IdRef => quote! {
                    &'a #ty {
                        &self.#graph[self.as_ref().#field]
                    }
                },
                AccessKind::IdWalker => quote! {
                    #ty<'a> {
                        self.as_ref().#field.walk(self.#graph)
                    }
                },
                AccessKind::ItemWalker => quote! {
                    #ty<'a> {
                        self.as_ref().#field.walk(self.#graph)
                    }
                },
                AccessKind::RefWalker => quote! {
                    #ty<'a> {
                        self.as_ref().#field.as_ref().walk(self.#graph)
                    }
                },
            },
            [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull] => match kind {
                AccessKind::Copy => quote! {
                    impl Iter<Item = #ty> + 'a {
                        self.as_ref().#field.iter().copied()
                    }
                },
                AccessKind::Ref => quote! {
                    impl Iter<Item = &'a #ty> + 'a {
                        self.as_ref().#field.iter()
                    }
                },
                AccessKind::IdRef => quote! {
                    impl Iter<Item = &'a #ty> + 'a {
                        self.as_ref().#field.walk(self.#graph)
                    }
                },
                AccessKind::IdWalker => {
                    quote! {
                        impl Iter<Item =  #ty<'a>> + 'a {
                            self.as_ref().#field.walk(self.#graph)
                        }
                    }
                }
                AccessKind::ItemWalker => quote! {
                    impl Iter<Item =  #ty<'a>> + 'a {
                        self.as_ref().#field.walk(self.#graph)
                    }
                },
                AccessKind::RefWalker => quote! {
                    impl Iter<Item = #ty<'a>> + 'a {
                        self.as_ref().#field.walk(self.#graph)
                    }
                },
            },
            [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull, WrappingType::List, WrappingType::NonNull] => {
                match kind {
                    AccessKind::IdRef if self.0.ty.name() != self.0.ty.walker_name() => quote! {
                        impl Iter<Item: Iter<Item = &'a #ty> + 'a> + 'a {
                            let #graph = self.#graph;
                            self.as_ref().#field.iter().map(move |items| items.iter().map(move |id| #graph[*id].as_ref()))
                        }
                    },
                    AccessKind::IdRef => quote! {
                        impl Iter<Item: Iter<Item = &'a #ty> + 'a> + 'a {
                            let #graph = self.#graph;
                            self.as_ref().#field.iter().map(move |items| items.iter().map(move |id| &#graph[*id]))
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

        tokens.append_all(quote! {
            #doc
            pub fn #method(&self) -> #return_type_and_body
        });
    }
}
