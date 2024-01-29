/// A change that represents a meaningful difference between the two schemas. Changes have a
/// direction: from source to target. For example, if `kind` is `AddField`, it means the field does
/// not exist in the `source` schema but it does exist in the `target` schema.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Change {
    // /!\ The order of fields matters for the PartialOrd derive /!\
    /// Where the change happened in the schema. It is dot separated where relevant. For example if
    /// the change happened in a field argument, the path will be something like
    /// `ParentTypeName.fieldName.argumentName`.
    pub path: String,
    /// The nature of the change.
    pub kind: ChangeKind,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
#[allow(missing_docs)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum ChangeKind {
    // /!\ The order of variants matters for the PartialOrd derive /!\
    ChangeQueryType,
    ChangeMutationType,
    ChangeSubscriptionType,
    RemoveObjectType,
    AddObjectType,
    AddInterfaceImplementation,
    RemoveInterfaceImplementation,
    ChangeFieldType,
    RemoveField,
    AddField,
    AddUnion,
    RemoveUnion,
    AddUnionMember,
    RemoveUnionMember,
    AddEnum,
    RemoveEnum,
    AddEnumValue,
    RemoveEnumValue,
    AddScalar,
    RemoveScalar,
    AddInterface,
    RemoveInterface,
    AddDirectiveDefinition,
    RemoveDirectiveDefinition,
    AddSchemaDefinition,
    RemoveSchemaDefinition,
    AddInputObject,
    RemoveInputObject,
    AddFieldArgument,
    RemoveFieldArgument,
    AddFieldArgumentDefault,
    RemoveFieldArgumentDefault,
    ChangeFieldArgumentDefault,
    ChangeFieldArgumentType,
}
