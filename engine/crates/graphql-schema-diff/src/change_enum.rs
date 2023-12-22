
pub struct Change {
    r#type: ChangeType,
}

pub enum ChangeType {
    AddedObjectType,
    RemovedObjectType,
    FieldTypeChanged
}
