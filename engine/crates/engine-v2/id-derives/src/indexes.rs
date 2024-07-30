use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, Field, MetaList};

pub fn derive_indexes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(fields),
        ..
    }) = input.data
    else {
        panic!("IndexImpls can only be derived on named field structs")
    };

    let indexed_fields = fields.named.iter().flat_map(parse_indexed_bys);

    let mut output = TokenStream::new();

    for field in indexed_fields {
        let id_type = &field.indexed_by;
        let field_name = field.field.ident.as_ref().unwrap();
        let field_ty = &field.field.ty;

        output.append_all(quote! {
            impl #impl_generics std::ops::Index<#id_type> for #ident #ty_generics #where_clause {
                type Output = <#field_ty as std::ops::Index<usize>>::Output;

                fn index(&self, index: #id_type) -> &Self::Output {
                    &self.#field_name[usize::from(index)]
                }
            }

            impl #impl_generics std::ops::IndexMut<#id_type> for #ident #ty_generics #where_clause {
                fn index_mut(&mut self, index: #id_type) -> &mut Self::Output {
                    &mut self.#field_name[usize::from(index)]
                }
            }

            impl #impl_generics std::ops::Index<id_newtypes::IdRange<#id_type>> for #ident #ty_generics #where_clause {
                type Output = [<#field_ty as std::ops::Index<usize>>::Output];

                fn index(&self, range: id_newtypes::IdRange<#id_type>) -> &Self::Output {
                    let id_newtypes::IdRange { start, end } = range;
                    let start = usize::from(start);
                    let end = usize::from(end);
                    &self.#field_name[start..end]
                }
            }

            impl #impl_generics std::ops::IndexMut<id_newtypes::IdRange<#id_type>> for #ident #ty_generics #where_clause {
                fn index_mut(&mut self, range: id_newtypes::IdRange<#id_type>) -> &mut Self::Output {
                    let id_newtypes::IdRange { start, end } = range;
                    let start = usize::from(start);
                    let end = usize::from(end);
                    &mut self.#field_name[start..end]
                }
            }
        });
    }

    proc_macro::TokenStream::from(output)
}

struct IndexedField<'a> {
    field: &'a Field,
    indexed_by: syn::TypePath,
}

fn parse_indexed_bys(field: &Field) -> impl Iterator<Item = IndexedField<'_>> + '_ {
    field
        .attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            syn::Meta::List(inner) if inner.path.is_ident("indexed_by") => Some(inner),
            _ => None,
        })
        .map(|meta_list| parse_indexed_by(field, meta_list))
}

fn parse_indexed_by<'a>(field: &'a Field, attribute: &MetaList) -> IndexedField<'a> {
    let indexed_by = attribute
        .parse_args::<syn::TypePath>()
        .expect("indexed_by takes a single type path");

    IndexedField { field, indexed_by }
}
