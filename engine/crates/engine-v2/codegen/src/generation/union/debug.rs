use proc_macro2::{Ident, Span};
use quote::{quote, TokenStreamExt};
use tracing::instrument;

use super::VariantContext;

pub struct DebugVariantBranch<'a> {
    pub variant: VariantContext<'a>,
    pub enum_name: &'a str,
}

impl quote::ToTokens for DebugVariantBranch<'_> {
    #[instrument(name = "debug_variant", skip_all, fields(variant = ?self.variant.variant))]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let enum_ = Ident::new(self.enum_name, Span::call_site());
        let variant = Ident::new(&self.variant.name, Span::call_site());
        let tt = if self.variant.value.is_some() {
            quote! {
                #enum_::#variant(variant) => variant.fmt(f),
            }
        } else {
            let name = proc_macro2::Literal::string(&self.variant.name);
            quote! {
                #enum_::#variant => write!(f, #name),
            }
        };

        tokens.append_all(tt);
    }
}
