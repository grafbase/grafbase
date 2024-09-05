use cynic_parser::common::WrappingType;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use tracing::instrument;

use crate::domain::{Domain, Object, ReaderKind};

use super::{debug::ReaderDebug, FieldContext};

#[instrument(skip_all)]
pub fn generate_reader(
    domain: &Domain,
    object: &Object,
    fields: &[FieldContext<'_>],
) -> anyhow::Result<Vec<TokenStream>> {
    let container_name = Ident::new(&domain.world_name, Span::call_site());
    let container_type = Ident::new(&domain.world_type_name, Span::call_site());
    let struct_name = Ident::new(&object.struct_name, Span::call_site());
    let reader_name = Ident::new(object.reader_name(), Span::call_site());
    let readable_trait = Ident::new(domain.readable_trait(), Span::call_site());

    let reader_field_methods = fields.iter().copied().map(ReaderFieldMethod);
    let mut code_sections = Vec::new();

    if let Some(indexed) = &object.indexed {
        let id_struct_name = Ident::new(&indexed.id_struct_name, Span::call_site());

        code_sections.push(quote! {
            #[derive(Clone, Copy)]
            pub struct #reader_name<'a> {
                #container_name: &'a #container_type,
                id: #id_struct_name,
            }
        });
        code_sections.push(quote! {
            impl <'a> #reader_name<'a> {
                #[allow(clippy::should_implement_trait)]
                pub fn as_ref(&self) -> &'a #struct_name {
                    &self.#container_name[self.id]
                }
                pub fn id(&self) -> #id_struct_name {
                    self.id
                }
                #(#reader_field_methods)*
            }
        });
        code_sections.push(quote! {
            impl #readable_trait<#container_type> for #id_struct_name {
                type Reader<'a> = #reader_name<'a>;

                fn read<'s>(self, #container_name: &'s #container_type) -> Self::Reader<'s>
                where
                    Self: 's
                {
                    #reader_name {
                        #container_name,
                        id: self,
                    }
                }
            }
        });
    } else if object.copy {
        code_sections.push(quote! {
            #[derive(Clone, Copy)]
            pub struct #reader_name<'a> {
                #container_name: &'a #container_type,
                item: #struct_name,
            }
        });
        code_sections.push(quote! {
            impl <'a> #reader_name<'a> {
                #[allow(clippy::should_implement_trait)]
                pub fn as_ref(&self) -> &#struct_name {
                    &self.item
                }
                #(#reader_field_methods)*
            }
        });
        code_sections.push(quote! {
            impl #readable_trait<#container_type> for #struct_name {
                type Reader<'a> = #reader_name<'a>;

                fn read<'s>(self, #container_name: &'s #container_type) -> Self::Reader<'s>
                where
                    Self: 's
                {
                    #reader_name {
                        #container_name,
                        item: self,
                    }
                }
            }
        });
    } else {
        code_sections.push(quote! {
            #[derive(Clone, Copy)]
            pub struct #reader_name<'a> {
                #container_name: &'a #container_type,
                ref_: &'a #struct_name,
            }
        });
        code_sections.push(quote! {
            impl <'a> #reader_name<'a> {
                #[allow(clippy::should_implement_trait)]
                pub fn as_ref(&self) -> &'a #struct_name {
                    self.ref_
                }
                #(#reader_field_methods)*
            }
        });
        code_sections.push(quote! {
            impl #readable_trait<#container_type> for &#struct_name {
                type Reader<'a> = #reader_name<'a>
                where
                    Self: 'a;

                fn read<'s>(self, #container_name: &'s #container_type) -> Self::Reader<'s>
                where
                    Self: 's
                {
                    #reader_name {
                        #container_name,
                        ref_: self,
                    }
                }
            }
        });
    }

    if object.meta.debug {
        code_sections.push(ReaderDebug { object, fields }.to_token_stream());
    }

    Ok(code_sections)
}

pub struct ReaderFieldMethod<'a>(FieldContext<'a>);

impl quote::ToTokens for ReaderFieldMethod<'_> {
    #[instrument(name = "reader_field_method", skip_all, fields(field = ?self.0.field))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let container = Ident::new(&self.0.domain.world_name, Span::call_site());
        let field = Ident::new(&self.0.struct_field_name(), Span::call_site());
        let method = Ident::new(&self.0.reader_method_name(), Span::call_site());
        let ty = Ident::new(self.0.ty.reader_name(), Span::call_site());
        let kind = self.0.ty.reader_kind();

        let method = match self.0.wrapping[..] {
            [] => match kind {
                ReaderKind::Copy => quote! {
                    pub fn #method(&self) -> Option<#ty> {
                        self.as_ref().#field
                    }
                },
                ReaderKind::Ref => quote! {
                    pub fn #method(&self) -> Option<&'a #ty> {
                        self.as_ref().#field.as_ref()
                    }
                },
                ReaderKind::IdRef if self.0.ty.name() != self.0.ty.reader_name() => quote! {
                    pub fn #method(&self) -> Option<&'a #ty> {
                        self.as_ref().#field.map(|id| self.#container[id].as_ref())
                    }
                },
                ReaderKind::IdRef => quote! {
                    pub fn #method(&self) -> Option<&'a #ty> {
                        self.as_ref().#field.map(|id| &self.#container[id])
                    }
                },
                ReaderKind::IdReader => quote! {
                    pub fn #method(&self) -> Option<#ty<'a>> {
                        self.as_ref().#field.as_ref().read(self.#container)
                    }
                },
                ReaderKind::ItemReader => quote! {
                    pub fn #method(&self) -> Option<#ty<'a>> {
                        self.as_ref().#field.as_ref().read(self.#container)
                    }
                },
                ReaderKind::RefReader => quote! {
                    pub fn #method(&self) -> Option<#ty<'a>> {
                        self.as_ref().#field.as_ref().read(self.#container)
                    }
                },
            },
            [WrappingType::NonNull] => match kind {
                ReaderKind::Copy => quote! {
                    pub fn #method(&self) -> #ty {
                        self.as_ref().#field
                    }
                },
                ReaderKind::Ref => quote! {
                    pub fn #method(&self) -> &'a #ty {
                        &self.as_ref().#field
                    }
                },
                ReaderKind::IdRef => quote! {
                    pub fn #method(&self) -> &'a #ty {
                        &self.#container[self.as_ref().#field]
                    }
                },
                ReaderKind::IdReader => quote! {
                    pub fn #method(&self) -> #ty<'a> {
                        self.as_ref().#field.read(self.#container)
                    }
                },
                ReaderKind::ItemReader => quote! {
                    pub fn #method(&self) -> #ty<'a> {
                        self.as_ref().#field.read(self.#container)
                    }
                },
                ReaderKind::RefReader => quote! {
                    pub fn #method(&self) -> #ty<'a> {
                        self.as_ref().#field.as_ref().read(self.#container)
                    }
                },
            },
            [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull] => match kind {
                ReaderKind::Copy => quote! {
                    pub fn #method(&self) -> impl Iter<Item = #ty> + 'a {
                        self.as_ref().#field.iter().copied()
                    }
                },
                ReaderKind::Ref => quote! {
                    pub fn #method(&self) -> impl Iter<Item = &'a #ty> + 'a {
                        self.as_ref().#field.iter()
                    }
                },
                ReaderKind::IdRef => quote! {
                    pub fn #method(&self) -> impl Iter<Item = &'a #ty> + 'a {
                        self.as_ref().#field.read(self.#container)
                    }
                },
                ReaderKind::IdReader => {
                    quote! {
                        pub fn #method(&self) -> impl Iter<Item =  #ty<'a>> + 'a {
                            self.as_ref().#field.read(self.#container)
                        }
                    }
                }
                ReaderKind::ItemReader => quote! {
                    pub fn #method(&self) -> impl Iter<Item =  #ty<'a>> + 'a {
                        self.as_ref().#field.read(self.#container)
                    }
                },
                ReaderKind::RefReader => quote! {
                    pub fn #method(&self) -> impl Iter<Item = #ty<'a>> + 'a {
                        self.as_ref().#field.read(self.#container)
                    }
                },
            },
            [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull, WrappingType::List, WrappingType::NonNull] => {
                match kind {
                    ReaderKind::IdRef if self.0.ty.name() != self.0.ty.reader_name() => quote! {
                        pub fn #method(&self) -> impl Iter<Item: Iter<Item = &'a #ty> + 'a> + 'a {
                            let #container = self.#container;
                            self.as_ref().#field.iter().map(move |items| items.iter().map(move |id| #container[*id].as_ref()))
                        }
                    },
                    ReaderKind::IdRef => quote! {
                        pub fn #method(&self) -> impl Iter<Item: Iter<Item = &'a #ty> + 'a> + 'a {
                            let #container = self.#container;
                            self.as_ref().#field.iter().map(move |items| items.iter().map(move |id| &#container[*id]))
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

        tokens.append_all(method);
    }
}
