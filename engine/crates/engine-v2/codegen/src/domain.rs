mod record;
mod scalar;
mod union;

use std::{collections::HashMap, path::PathBuf};

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
pub use record::*;
pub use scalar::*;
pub use union::*;

#[derive(Debug)]
pub struct Domain {
    pub name: String,
    pub source: PathBuf,
    pub sdl: String,
    pub destination_path: PathBuf,
    pub module: TokenStream,
    pub public_visibility: TokenStream,
    pub context_name: String,
    pub context_type: TokenStream,
    pub definitions_by_name: HashMap<String, Definition>,
    pub imported_domains: HashMap<String, ImportedDomain>,
}

impl Domain {
    pub fn context_accessor(&self, external_domain_name: Option<&str>) -> TokenStream {
        if let Some(name) = external_domain_name {
            let domain = Ident::new(&self.imported_domains[name].name, Span::call_site());
            let ctx = Ident::new(&self.context_name, Span::call_site());
            quote! { #ctx.#domain }
        } else {
            let ctx = Ident::new(&self.context_name, Span::call_site());
            quote! { #ctx }
        }
    }

    pub fn domain_accessor(&self, external_domain_name: Option<&str>) -> TokenStream {
        if let Some(name) = external_domain_name {
            let domain = Ident::new(&self.imported_domains[name].name, Span::call_site());
            let ctx = Ident::new(&self.context_name, Span::call_site());
            quote! { #ctx.#domain }
        } else if self.name != self.context_name {
            let domain = Ident::new(&self.name, Span::call_site());
            let ctx = Ident::new(&self.context_name, Span::call_site());
            quote! { #ctx.#domain }
        } else {
            let domain = Ident::new(&self.name, Span::call_site());
            quote! { #domain }
        }
    }
}

#[derive(Debug)]
pub struct ImportedDomain {
    pub name: String,
    pub module: TokenStream,
}

#[derive(Debug)]
pub enum Definition {
    Scalar(Scalar),
    Object(Object),
    Union(Union),
}

impl Definition {
    pub fn set_external_domain_name(&mut self, name: String) {
        match self {
            Definition::Scalar(scalar) => {
                scalar.external_domain_name = Some(name);
            }
            Definition::Object(object) => {
                object.external_domain_name = Some(name);
            }
            Definition::Union(union) => {
                union.external_domain_name = Some(name);
            }
        }
    }

    pub fn external_domain_name(&self) -> Option<&str> {
        match self {
            Definition::Scalar(scalar) => scalar.external_domain_name.as_deref(),
            Definition::Object(object) => object.external_domain_name.as_deref(),
            Definition::Union(union) => union.external_domain_name.as_deref(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Definition::Scalar(scalar) => &scalar.name,
            Definition::Object(object) => &object.name,
            Definition::Union(union) => union.name(),
        }
    }

    pub fn span(&self) -> &cynic_parser::Span {
        match self {
            Definition::Scalar(scalar) => &scalar.span,
            Definition::Object(object) => &object.span,
            Definition::Union(union) => &union.span,
        }
    }

    pub fn walker_name(&self) -> &str {
        match self {
            Definition::Scalar(scalar) => scalar.walker_name(),
            Definition::Object(object) => object.walker_name(),
            Definition::Union(union) => union.walker_name(),
        }
    }

    pub fn storage_type(&self) -> StorageType {
        match self {
            Definition::Scalar(scalar) => {
                if let Some(indexed) = &scalar.indexed {
                    StorageType::Id {
                        name: &indexed.id_struct_name,
                        list_as_id_range: !indexed.deduplicated,
                    }
                } else {
                    StorageType::Struct {
                        name: &scalar.struct_name,
                    }
                }
            }
            Definition::Object(object) => {
                if let Some(indexed) = &object.indexed {
                    StorageType::Id {
                        name: &indexed.id_struct_name,
                        list_as_id_range: !indexed.deduplicated,
                    }
                } else {
                    StorageType::Struct {
                        name: &object.struct_name,
                    }
                }
            }
            Definition::Union(union) => {
                if let Some(indexed) = union.indexed() {
                    StorageType::Id {
                        name: &indexed.id_struct_name,
                        list_as_id_range: !indexed.deduplicated,
                    }
                } else {
                    StorageType::Struct {
                        name: union.enum_name(),
                    }
                }
            }
        }
    }

    pub fn access_kind(&self) -> AccessKind {
        match self {
            Definition::Scalar(scalar) => {
                if scalar.is_record {
                    if scalar.copy {
                        AccessKind::ItemWalker
                    } else if scalar.indexed.is_some() {
                        AccessKind::IdWalker
                    } else {
                        AccessKind::RefWalker
                    }
                } else if scalar.copy {
                    AccessKind::Copy
                } else if scalar.indexed.is_some() {
                    AccessKind::IdRef
                } else {
                    AccessKind::Ref
                }
            }
            Definition::Object(record) => {
                if record.indexed.is_some() {
                    AccessKind::IdWalker
                } else if record.copy {
                    AccessKind::ItemWalker
                } else {
                    AccessKind::RefWalker
                }
            }
            Definition::Union(union) => match &union.kind {
                UnionKind::Record(record) => {
                    if record.indexed.is_some() {
                        AccessKind::IdWalker
                    } else if record.copy {
                        AccessKind::ItemWalker
                    } else {
                        AccessKind::RefWalker
                    }
                }
                UnionKind::Id(_) | UnionKind::BitpackedId(_) => AccessKind::ItemWalker,
            },
        }
    }
}

#[derive(Debug)]
pub enum StorageType<'a> {
    Id { name: &'a str, list_as_id_range: bool },
    Struct { name: &'a str },
}

impl<'a> StorageType<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            StorageType::Id { name, .. } => name,
            StorageType::Struct { name } => name,
        }
    }

    pub fn is_id(&self) -> bool {
        match self {
            StorageType::Id { .. } => true,
            StorageType::Struct { .. } => false,
        }
    }

    pub fn list_as_id_range(&self) -> bool {
        match self {
            StorageType::Id { list_as_id_range, .. } => *list_as_id_range,
            StorageType::Struct { .. } => false,
        }
    }
}

impl std::fmt::Display for StorageType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StorageType::Id { name, .. } => {
                write!(f, "{}", name)
            }
            StorageType::Struct { name } => write!(f, "{}", name),
        }
    }
}

#[derive(Debug)]
pub enum AccessKind {
    Copy,
    Ref,
    IdRef,
    IdWalker,
    RefWalker,
    ItemWalker,
}

#[derive(Default, Debug)]
pub struct Meta {
    pub module_path: Vec<String>,
    pub derive: Vec<String>,
    pub debug: bool,
}

#[derive(Debug)]
pub struct Indexed {
    pub id_struct_name: String,
    pub id_size: Option<String>,
    pub max_id: Option<String>,
    pub deduplicated: bool,
}
