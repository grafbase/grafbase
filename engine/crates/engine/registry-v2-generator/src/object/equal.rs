use proc_macro2::Ident;
use quote::{quote, ToTokens, TokenStreamExt};

use super::FieldEdge;

#[allow(dead_code)]
pub struct ObjectEqual<'a>(pub &'a Ident, pub &'a [FieldEdge<'a>]);

impl ToTokens for ObjectEqual<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ObjectEqual(reader_name, _) = self;

        tokens.append_all(quote! {
            impl std::cmp::PartialEq for #reader_name<'_> {
                fn eq(&self, other: &#reader_name<'_>) -> bool {
                    // We could generate comparisons for the contents, but for now I'm just
                    // going to go with comparing IDs and assuming that means they're different
                    std::ptr::eq(self.0.registry, other.0.registry) && self.0.id == other.0.id
                }
            }

            impl std::cmp::Eq for #reader_name<'_> {}
        });
    }
}
