mod id;
mod indexes;

#[proc_macro_derive(Id, attributes(max))]
pub fn derive_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    self::id::derive_id(input)
}

#[proc_macro_derive(IndexedFields, attributes(indexed_by))]
pub fn derive_indexes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    indexes::derive_indexes(input)
}
