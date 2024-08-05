mod id;
mod indexes;

#[proc_macro_attribute]
pub fn id(attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    self::id::add_derive(attr, input)
}

#[proc_macro_derive(Id, attributes(max))]
pub fn derive_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    self::id::derive_id(input)
}

#[proc_macro_derive(IndexedFields, attributes(indexed_by))]
pub fn derive_indexes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    indexes::derive_indexes(input)
}
