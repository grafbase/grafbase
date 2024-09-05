use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt};
use tracing::instrument;

use crate::domain::{Definition, Domain, ReaderKind, Union, UnionKind};

use super::VariantContext;

#[instrument(skip_all)]
pub fn generate_reader(
    domain: &Domain,
    union: &Union,
    variants: &[VariantContext<'_>],
) -> anyhow::Result<Vec<TokenStream>> {
    let enum_name = Ident::new(union.enum_name(), Span::call_site());
    let container_name = Ident::new(&domain.world_name, Span::call_site());
    let container_type = Ident::new(&domain.world_type_name, Span::call_site());
    let reader_enum_name = Ident::new(union.reader_enum_name(), Span::call_site());
    let readable_trait = Ident::new(domain.readable_trait(), Span::call_site());

    let mut code_sections = Vec::new();

    let reader_variants = variants.iter().copied().map(ReaderVariant);
    code_sections.push(quote! {
        #[derive(Clone, Copy, Debug)]
        pub enum #reader_enum_name<'a> {
            #(#reader_variants),*
        }
    });

    match &union.kind {
        UnionKind::Record(record) => {
            if let Some(indexed) = &record.indexed {
                let reader_name = Ident::new(union.reader_name(), Span::call_site());
                let id_struct_name = Ident::new(&indexed.id_struct_name, Span::call_site());

                code_sections.insert(
                    0,
                    quote! {
                        #[derive(Clone, Copy)]
                        pub struct #reader_name<'a> {
                            pub(crate) #container_name: &'a #container_type,
                            pub(crate) id: #id_struct_name,
                        }
                    },
                );
                code_sections.push(quote! {
                    impl std::ops::Deref for #reader_name<'_> {
                        type Target = #enum_name;
                        fn deref(&self) -> &Self::Target {
                            self.as_ref()
                        }
                    }
                });

                let read_branches = variants.iter().copied().map(|variant| RecordUnionReaderBranch {
                    variant,
                    reader_enum_name: union.reader_enum_name(),
                    enum_name: union.enum_name(),
                });
                code_sections.push(quote! {
                    impl<'a> #reader_name<'a> {
                        #[allow(clippy::should_implement_trait)]
                        pub fn as_ref(&self) -> &'a #enum_name {
                            &self.#container_name[self.id]
                        }
                        pub fn id(&self) -> #id_struct_name {
                            self.id
                        }
                        pub fn variant(&self) -> #reader_enum_name<'a> {
                            let #container_name = self.#container_name;
                            match self.as_ref() {
                                #(#read_branches),*
                            }
                        }
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

                code_sections.push(quote! {
                    impl std::fmt::Debug for #reader_name<'_> {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            self.variant().fmt(f)
                        }
                    }
                });
            } else {
                tracing::error!("Could not generate a reader, it's neither an @id nor @indexed.",);
                unimplemented!()
            }
        }
        UnionKind::Id(_) | UnionKind::BitpackedId(_) => {
            let read_branches = variants.iter().copied().map(|variant| IdUnionReaderBranch {
                variant,
                reader_enum_name: union.reader_enum_name(),
                enum_name: union.enum_name(),
            });

            code_sections.push(quote! {
                impl #readable_trait<#container_type> for #enum_name {
                    type Reader<'a> = #reader_enum_name<'a>;

                    fn read<'s>(self, #container_name: &'s #container_type) -> Self::Reader<'s>
                    where
                        Self: 's
                    {
                        match self {
                            #(#read_branches),*
                        }
                    }
                }
            });

            match variants
                .iter()
                .copied()
                .map(|variant| {
                    IdUnionReaderIdMethodBranch {
                        variant,
                        reader_enum_name: union.reader_enum_name(),
                        enum_name: union.enum_name(),
                    }
                    .try_to_tokens()
                })
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(id_branches) => {
                    code_sections.push(quote! {
                        impl #reader_enum_name<'_> {
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
                        "Could not generate id() method for reader '{}' because variant '{}' has a value '{}' which doesn't have any.",
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

struct ReaderVariant<'a>(VariantContext<'a>);

impl quote::ToTokens for ReaderVariant<'_> {
    #[instrument(name = "reader_variant", skip_all, fields(variant = ?self.0.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant = Ident::new(&self.0.name, Span::call_site());
        let tt = if let Some(value) = self.0.value {
            let reader = Ident::new(value.reader_name(), Span::call_site());
            match value {
                Definition::Scalar(scalar) if !scalar.is_record => {
                    if scalar.copy {
                        quote! { #variant(#reader) }
                    } else {
                        quote! { #variant(&'a #reader) }
                    }
                }
                _ => {
                    quote! { #variant(#reader<'a>) }
                }
            }
        } else {
            quote! { #variant }
        };
        tokens.append_all(tt);
    }
}

struct RecordUnionReaderBranch<'a> {
    variant: VariantContext<'a>,
    enum_name: &'a str,
    reader_enum_name: &'a str,
}

impl quote::ToTokens for RecordUnionReaderBranch<'_> {
    #[instrument(name = "record_union_reader_branch", skip_all, fields(variant = ?self.variant.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let enum_ = Ident::new(self.enum_name, Span::call_site());
        let variant = Ident::new(&self.variant.name, Span::call_site());
        let reader = Ident::new(self.reader_enum_name, Span::call_site());

        let tt = if let Some(value) = self.variant.value {
            let container = Ident::new(&self.variant.domain.world_name, Span::call_site());
            match value.reader_kind() {
                ReaderKind::Copy => {
                    quote! { #enum_::#variant(item) => #reader::#variant(item) }
                }
                ReaderKind::Ref => {
                    quote! { #enum_::#variant(item) => #reader::#variant(&item) }
                }
                ReaderKind::IdRef => {
                    quote! { #enum_::#variant(id) => #reader::#variant(&#container[id]) }
                }
                ReaderKind::IdReader => {
                    quote! { #enum_::#variant(id) => #reader::#variant(id.read(#container)) }
                }
                ReaderKind::ItemReader => {
                    quote! { #enum_::#variant(item) => #reader::#variant(item.read(#container)) }
                }
                ReaderKind::RefReader => {
                    quote! { #enum_::#variant(ref item) => #reader::#variant(item.read(#container)) }
                }
            }
        } else {
            quote! { #enum_::#variant => #reader::#variant }
        };
        tokens.append_all(tt);
    }
}

struct IdUnionReaderBranch<'a> {
    variant: VariantContext<'a>,
    enum_name: &'a str,
    reader_enum_name: &'a str,
}

impl quote::ToTokens for IdUnionReaderBranch<'_> {
    #[instrument(name = "id_union_reader_branch", skip_all, fields(variant = ?self.variant.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let enum_ = Ident::new(self.enum_name, Span::call_site());
        let reader = Ident::new(self.reader_enum_name, Span::call_site());
        let variant = Ident::new(&self.variant.name, Span::call_site());
        let container = Ident::new(&self.variant.domain.world_name, Span::call_site());

        let tt = match self.variant.value {
            Some(Definition::Scalar(scalar)) if !scalar.is_record => {
                quote! {
                    #enum_::#variant(id) => #reader::#variant(&#container[id])
                }
            }
            Some(value) => {
                if value.storage_type().is_id() {
                    quote! {
                        #enum_::#variant(id) => #reader::#variant(id.read(#container))
                    }
                } else {
                    quote! {
                        #enum_::#variant(item) => #reader::#variant(item.read(#container))
                    }
                }
            }
            _ => {
                quote! {
                    #enum_::#variant => #reader::#variant
                }
            }
        };
        tokens.append_all(tt);
    }
}

struct IdUnionReaderIdMethodBranch<'a> {
    variant: VariantContext<'a>,
    enum_name: &'a str,
    reader_enum_name: &'a str,
}

impl<'a> IdUnionReaderIdMethodBranch<'a> {
    fn try_to_tokens(&self) -> Result<TokenStream, (&'a str, &'a str)> {
        let enum_ = Ident::new(self.enum_name, Span::call_site());
        let reader = Ident::new(self.reader_enum_name, Span::call_site());
        let variant = Ident::new(&self.variant.name, Span::call_site());

        let tt = match self.variant.value {
            Some(Definition::Scalar(scalar)) => {
                if scalar.is_record && scalar.indexed.is_some() {
                    quote! {
                        #reader::#variant(reader) => #enum_::#variant(reader.id)
                    }
                } else if scalar.copy {
                    quote! {
                        #reader::#variant(item) => #enum_::#variant(item)
                    }
                } else {
                    return Err((&self.variant.variant.name, &scalar.name));
                }
            }
            Some(Definition::Object(object)) => {
                if object.indexed.is_some() {
                    quote! {
                        #reader::#variant(reader) => #enum_::#variant(reader.id)
                    }
                } else if object.copy {
                    quote! {
                        #reader::#variant(reader) => #enum_::#variant(reader.item)
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
                    #reader::#variant => #enum_::#variant
                }
            }
        };

        Ok(tt)
    }
}
