use quote::{TokenStreamExt, quote};
use syn::parse_macro_input;

pub fn derive_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Unnamed(fields),
        ..
    }) = input.data
    else {
        panic!("IndexImpls can only be derived on named field structs")
    };

    let too_many_error = proc_macro2::Literal::string(&format!("Too many {ident}"));
    let ident_string = ident.to_string();
    let stripped_name = ident_string.strip_suffix("Id").unwrap_or(&ident_string);
    let name_format_str = proc_macro2::Literal::string(&format!("{stripped_name}#{{}}"));

    let mut output = match find_non_zero_kind(&fields) {
        Some(inner_ty) => quote! {
            impl #impl_generics From<usize> for #ident #ty_generics #where_clause {
                fn from(value: usize) -> Self {
                    let value = #inner_ty::try_from(value).expect(#too_many_error);
                    Self(
                        (value + 1).try_into().expect(#too_many_error)
                    )
                }
            }

            impl #impl_generics From<#inner_ty> for #ident #ty_generics #where_clause {
                fn from(value: #inner_ty) -> Self {
                    Self(
                        (value + 1).try_into().expect(#too_many_error)
                    )
                }
            }

            impl From<#ident> for usize {
                fn from(id: #ident) -> Self {
                    ((id.0.get() - 1) as usize)
                }
            }

            impl From<#ident> for #inner_ty {
                fn from(id: #ident) -> Self {
                    (id.0.get() - 1)
                }
            }
        },
        None => {
            let inner_ty = &fields.unnamed.first().expect("Empty tuple?").ty;
            quote! {
                impl #impl_generics From<usize> for #ident #ty_generics #where_clause {
                    fn from(value: usize) -> Self {
                        let value = #inner_ty::try_from(value).expect(#too_many_error);
                        Self(value)
                    }
                }

                impl #impl_generics From<#inner_ty> for #ident #ty_generics #where_clause {
                    fn from(value: #inner_ty) -> Self {
                        Self(value)
                    }
                }

                impl From<#ident> for usize {
                    fn from(id: #ident) -> Self {
                        id.0 as usize
                    }
                }

                impl From<#ident> for #inner_ty {
                    fn from(id: #ident) -> Self {
                        id.0
                    }
                }
            }
        }
    };

    output.append_all(quote! {
        impl std::fmt::Display for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, #name_format_str, usize::from(*self))
            }
        }
        impl std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, #name_format_str, usize::from(*self))
            }
        }
    });

    proc_macro::TokenStream::from(output)
}

/// Finds the u8 in NonZero<u8>
fn find_non_zero_kind(fields: &syn::FieldsUnnamed) -> Option<&syn::Type> {
    let syn::Type::Path(path) = &fields.unnamed.first()?.ty else {
        return None;
    };

    let last_segment = path.path.segments.last()?;

    match &last_segment.arguments {
        syn::PathArguments::AngleBracketed(params) => params.args.iter().find_map(|arg| match arg {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        }),
        _ => None,
    }
}
