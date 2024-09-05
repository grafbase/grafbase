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
    let container_name = Ident::new(&domain.world_name, Span::call_site());
    let container_type = Ident::new(&domain.world_type_name, Span::call_site());
    let reader_enum_name = Ident::new(union.reader_name(), Span::call_site());
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
                let id_struct_name = Ident::new(&indexed.id_struct_name, Span::call_site());
                let enum_name = &union.enum_name();
                let reader_enum_name = &union.reader_name();
                let read_branches = variants.iter().copied().map(|variant| RecordUnionReaderBranch {
                    variant,
                    reader_enum_name,
                    enum_name,
                });
                let reader_enum_name = Ident::new(reader_enum_name, Span::call_site());
                code_sections.push(quote! {
                    impl #readable_trait<#container_type> for #id_struct_name {
                        type Reader<'a> = #reader_enum_name<'a>;

                        fn read<'s>(self, #container_name: &'s #container_type) -> Self::Reader<'s>
                        where
                            Self: 's
                        {
                            match #container_name[self] {
                                #(#read_branches),*
                            }
                        }
                    }
                });
            } else {
                tracing::error!("Could not generate a reader, it's neither an @id nor @indexed.",);
                unimplemented!()
            }
        }
        UnionKind::Id(_) | UnionKind::BitpackedId(_) => {
            let enum_name = &union.enum_name();
            let reader_enum_name = &union.reader_name();
            let read_branches = variants.iter().copied().map(|variant| IdUnionReaderBranch {
                variant,
                reader_enum_name,
                enum_name,
            });

            let reader_enum_name = Ident::new(reader_enum_name, Span::call_site());
            let enum_name = Ident::new(union.enum_name(), Span::call_site());
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
                Definition::Scalar(scalar) if !scalar.has_custom_reader => {
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
            Some(Definition::Scalar(scalar)) if !scalar.has_custom_reader => {
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
