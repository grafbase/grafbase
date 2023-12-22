#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub struct Change {
    // /!\ The order of fields matters for the PartialOrd derive /!\
    pub path: String,
    pub kind: ChangeKind,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum ChangeKind {
    // /!\ The order of variants matters for the PartialOrd derive /!\
    RemovedObjectType,
    AddedObjectType,
    FieldTypeChanged,
}
