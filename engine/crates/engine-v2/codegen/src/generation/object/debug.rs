use cynic_parser::common::WrappingType;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens, TokenStreamExt};
use tracing::instrument;

use crate::domain::{Object, ReaderKind};

use super::FieldContext;

#[derive(Clone, Copy)]
pub struct ReaderDebug<'a> {
    pub object: &'a Object,
    pub fields: &'a [FieldContext<'a>],
}

impl ToTokens for ReaderDebug<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ReaderDebug { object, fields } = *self;

        let reader_struct = Ident::new(object.reader_name(), Span::call_site());
        let name_string = proc_macro2::Literal::string(object.reader_name());

        let fields = fields.iter().copied().map(DebugField);

        tokens.append_all(quote! {
            impl std::fmt::Debug for #reader_struct<'_> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(#name_string)
                        #(#fields)*.finish()
                }
            }
        });
    }
}

pub struct DebugField<'a>(FieldContext<'a>);

impl ToTokens for DebugField<'_> {
    #[instrument(name = "debug", skip_all, fields(field = ?self.0.field))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let DebugField(field) = self;

        let name = Ident::new(&field.name, Span::call_site());
        let name_string = proc_macro2::Literal::string(&field.name);
        let kind = self.0.ty.reader_kind();

        let tt = match kind {
            ReaderKind::Copy | ReaderKind::Ref | ReaderKind::IdRef => match field.wrapping[..] {
                [] | [WrappingType::NonNull] => {
                    quote! { .field(#name_string, &self.#name()) }
                }
                [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull]
                | [WrappingType::NonNull, WrappingType::List] => {
                    quote! { .field(#name_string, &self.#name()).collect::<Vec<_>>() }
                }
                [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull, WrappingType::List, WrappingType::NonNull]
                | [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull, WrappingType::List] => {
                    quote! { .field(#name_string, &self.#name().map(|items| items.collect::<Vec<_>>()).collect::<Vec<_>>()) }
                }
                _ => {
                    tracing::error!("Unsupported wrapping {:?}", self.0.wrapping);
                    unimplemented!()
                }
            },
            ReaderKind::IdReader | ReaderKind::RefReader | ReaderKind::ItemReader => {
                quote! { .field(#name_string, &self.#name()) }
            }
        };

        tokens.append_all(tt);
    }
}
