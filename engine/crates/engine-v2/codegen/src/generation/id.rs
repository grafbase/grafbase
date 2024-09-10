use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::domain::{Domain, Indexed};

pub fn generate_id(_domain: &Domain, indexed: &Indexed) -> anyhow::Result<Vec<TokenStream>> {
    let id_struct_name = Ident::new(&indexed.id_struct_name, Span::call_site());

    let id_struct = if let Some(size) = &indexed.id_size {
        let size = Ident::new(size, Span::call_site());
        if let Some(max_id) = &indexed.max_id {
            let max_id = Ident::new(max_id, Span::call_site());
            quote! {
                #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
                #[max(#max_id)]
                pub struct #id_struct_name(std::num::NonZero<#size>);
            }
        } else {
            quote! {
                #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
                pub struct #id_struct_name(std::num::NonZero<#size>);
            }
        }
    } else {
        quote! {}
    };

    Ok(vec![id_struct])
}
