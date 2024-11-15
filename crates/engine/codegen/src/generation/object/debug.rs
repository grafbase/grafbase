use cynic_parser::common::WrappingType;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens, TokenStreamExt};
use tracing::instrument;

use crate::domain::{AccessKind, Object};

use super::FieldContext;

#[derive(Clone, Copy)]
pub struct WalkerDebug<'a> {
    pub object: &'a Object,
    pub fields: &'a [FieldContext<'a>],
}

impl ToTokens for WalkerDebug<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let WalkerDebug { object, fields } = *self;

        let walker_struct = Ident::new(object.walker_name(), Span::call_site());
        let name_string = proc_macro2::Literal::string(object.walker_name());

        let fields = fields.iter().map(DebugField);

        tokens.append_all(quote! {
            impl std::fmt::Debug for #walker_struct<'_> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(#name_string)
                        #(#fields)*.finish()
                }
            }
        });
    }
}

pub struct DebugField<'a>(&'a FieldContext<'a>);

impl ToTokens for DebugField<'_> {
    #[instrument(name = "debug", skip_all, fields(field = ?self.0.field))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let DebugField(field) = self;

        let name = Ident::new(&field.name, Span::call_site());
        let name_string = proc_macro2::Literal::string(&field.name);
        let kind = self.0.ty.access_kind();

        let tt = if field.has_walker {
            match kind {
                AccessKind::Copy | AccessKind::Ref | AccessKind::IdRef => match field.wrapping[..] {
                    [] | [WrappingType::NonNull] => {
                        quote! { .field(#name_string, &self.#name()) }
                    }
                    [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull]
                    | [WrappingType::NonNull, WrappingType::List] => {
                        quote! { .field(#name_string, &self.#name()) }
                    }
                    [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull, WrappingType::List, WrappingType::NonNull]
                    | [WrappingType::NonNull, WrappingType::List, WrappingType::NonNull, WrappingType::List] => {
                        quote! { .field(#name_string, &self.#name().map(|items| items.collect::<Vec<_>>())) }
                    }
                    _ => {
                        tracing::error!("Unsupported wrapping {:?}", self.0.wrapping);
                        unimplemented!()
                    }
                },
                AccessKind::IdWalker | AccessKind::RefWalker | AccessKind::ItemWalker => {
                    quote! { .field(#name_string, &self.#name()) }
                }
            }
        } else {
            quote! { .field(#name_string, &self.#name) }
        };

        tokens.append_all(tt);
    }
}
