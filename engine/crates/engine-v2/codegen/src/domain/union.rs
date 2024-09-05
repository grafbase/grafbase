use super::{Definition, Indexed, Meta};

#[derive(Debug)]
pub struct Union {
    pub meta: Meta,
    pub kind: UnionKind,
    pub span: cynic_parser::Span,
    pub variants: Vec<Variant>,
}

#[derive(Debug)]
pub struct Variant {
    pub name: String,
    pub index: usize,
    pub value_type_name: Option<String>,
}

impl From<Union> for Definition {
    fn from(union: Union) -> Self {
        Definition::Union(union)
    }
}

impl Union {
    pub fn name(&self) -> &str {
        match &self.kind {
            UnionKind::Record(record_union) => &record_union.name,
            UnionKind::Id(id_union) => &id_union.name,
            UnionKind::BitpackedId(bitpacked_id_union) => &bitpacked_id_union.name,
        }
    }

    pub fn indexed(&self) -> Option<&Indexed> {
        match &self.kind {
            UnionKind::Record(record_union) => record_union.indexed.as_ref(),
            UnionKind::Id(_) | UnionKind::BitpackedId(_) => None,
        }
    }

    pub fn enum_name(&self) -> &str {
        match &self.kind {
            UnionKind::Record(record_union) => &record_union.enum_name,
            UnionKind::Id(id_union) => &id_union.enum_name,
            UnionKind::BitpackedId(bitpacked_id_union) => &bitpacked_id_union.enum_name,
        }
    }

    pub fn reader_name(&self) -> &str {
        match &self.kind {
            UnionKind::Record(record_union) => record_union.read_name(),
            UnionKind::Id(id_union) => id_union.reader_name(),
            UnionKind::BitpackedId(bitpacked_id_union) => bitpacked_id_union.reader_name(),
        }
    }

    pub fn reader_enum_name(&self) -> &str {
        match &self.kind {
            UnionKind::Record(record_union) => record_union.reader_enum_name(),
            UnionKind::Id(id_union) => id_union.reader_name(),
            UnionKind::BitpackedId(bitpacked_id_union) => bitpacked_id_union.reader_name(),
        }
    }
}

#[derive(Debug)]
pub enum UnionKind {
    Record(RecordUnion),
    Id(IdUnion),
    BitpackedId(BitPackedIdUnion),
}

#[derive(Debug)]
pub struct RecordUnion {
    pub indexed: Option<Indexed>,
    pub copy: bool,
    pub name: String,
    pub reader_enum_name: String,
    pub enum_name: String,
}

impl RecordUnion {
    pub fn read_name(&self) -> &str {
        &self.name
    }

    pub fn reader_enum_name(&self) -> &str {
        &self.reader_enum_name
    }
}

#[derive(Debug)]
pub struct IdUnion {
    pub name: String,
    pub enum_name: String,
}

impl IdUnion {
    pub fn reader_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct BitPackedIdUnion {
    pub size: String,
    pub name: String,
    pub enum_name: String,
}

impl BitPackedIdUnion {
    pub fn reader_name(&self) -> &str {
        &self.name
    }

    pub fn bitpacked_enum_name(&self) -> String {
        format!("BitPacked{}Id", self.name)
    }
}
