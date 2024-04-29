use proc_macro2::{Ident, Span};
use quote::{quote, TokenStreamExt};

#[derive(Clone, Copy)]
pub struct IdIdent<'a>(pub &'a str);

impl quote::ToTokens for IdIdent<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = Ident::new(&format!("{}Id", self.0), Span::call_site());

        tokens.append_all(quote! { #ident })
    }
}
