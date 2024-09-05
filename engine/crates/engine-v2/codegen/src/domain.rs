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
    pub world_name: String,
    pub world_type_name: String,
    pub definitions_by_name: HashMap<String, Definition>,
}

impl Domain {
    pub fn readable_trait(&self) -> &str {
        "Readable"
    }
}

#[derive(Debug)]
pub enum Definition {
    Scalar(Scalar),
    Object(Object),
    Union(Union),
}

impl Definition {
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

    pub fn reader_name(&self) -> &str {
        match self {
            Definition::Scalar(scalar) => scalar.reader_name(),
            Definition::Object(object) => object.reader_name(),
            Definition::Union(union) => union.reader_name(),
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

    pub fn reader_kind(&self) -> ReaderKind {
        match self {
            Definition::Scalar(scalar) => {
                if scalar.is_record {
                    if scalar.copy {
                        ReaderKind::ItemReader
                    } else if scalar.indexed.is_some() {
                        ReaderKind::IdReader
                    } else {
                        ReaderKind::RefReader
                    }
                } else if scalar.copy {
                    ReaderKind::Copy
                } else if scalar.indexed.is_some() {
                    ReaderKind::IdRef
                } else {
                    ReaderKind::Ref
                }
            }
            Definition::Object(record) => {
                if record.indexed.is_some() {
                    ReaderKind::IdReader
                } else if record.copy {
                    ReaderKind::ItemReader
                } else {
                    ReaderKind::RefReader
                }
            }
            Definition::Union(union) => match &union.kind {
                UnionKind::Record(record) => {
                    if record.indexed.is_some() {
                        ReaderKind::IdReader
                    } else if record.copy {
                        ReaderKind::ItemReader
                    } else {
                        ReaderKind::RefReader
                    }
                }
                UnionKind::Id(_) | UnionKind::BitpackedId(_) => ReaderKind::ItemReader,
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
pub enum ReaderKind {
    Copy,
    Ref,
    IdRef,
    IdReader,
    RefReader,
    ItemReader,
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
