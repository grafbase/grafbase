use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens, TokenStreamExt};

use super::FieldEdge;

pub struct ObjectDebug<'a>(pub &'a Ident, pub &'a [FieldEdge<'a>]);

impl ToTokens for ObjectDebug<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ObjectDebug(reader_name, fields) = self;

        let name_string = proc_macro2::Literal::string(&reader_name.to_string());

        let fields = fields.iter().copied().map(DebugField);

        tokens.append_all(quote! {
            impl fmt::Debug for #reader_name<'_> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.debug_struct(#name_string)
                        #(#fields)*.finish()
                }
            }
        });
    }
}

pub struct DebugField<'a>(FieldEdge<'a>);

impl ToTokens for DebugField<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let DebugField(edge) = self;

        let name_string = edge.field.name();
        let name = Ident::new(name_string, Span::call_site());
        let name_string = proc_macro2::Literal::string(name_string);

        if edge.field.ty().is_list() {
            tokens.append_all(quote! { .field(#name_string, &self.#name().collect::<Vec<_>>()) });
        } else {
            tokens.append_all(quote! { .field(#name_string, &self.#name()) });
        }
    }
}
