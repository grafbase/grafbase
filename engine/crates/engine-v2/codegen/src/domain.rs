mod record;
mod scalar;
mod union;

use std::{collections::HashMap, path::PathBuf};

pub use record::*;
pub use scalar::*;
pub use union::*;

#[derive(Debug)]
pub struct Domain {
    pub source: PathBuf,
    pub sdl: String,
    pub destination_path: PathBuf,
    pub root_module: Vec<String>,
    pub graph_var_name: String,
    pub graph_type_name: String,
    pub definitions_by_name: HashMap<String, Definition>,
}

#[derive(Debug)]
pub enum Definition {
    Scalar(Scalar),
    Object(Object),
    Union(Union),
}

impl Definition {
    pub fn is_scalar(&self) -> bool {
        matches!(self, Definition::Scalar(_))
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
pub struct FieldMeta {
    pub debug: bool,
}

impl Default for FieldMeta {
    fn default() -> Self {
        Self { debug: true }
    }
}

#[derive(Debug)]
pub struct Indexed {
    pub id_struct_name: String,
    pub id_size: Option<String>,
    pub max_id: Option<String>,
    pub deduplicated: bool,
}
