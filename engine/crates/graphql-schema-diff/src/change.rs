#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Change {
    // /!\ The order of fields matters for the PartialOrd derive /!\
    pub path: String,
    pub kind: ChangeKind,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum ChangeKind {
    // /!\ The order of variants matters for the PartialOrd derive /!\
    ChangeQueryType,
    ChangeMutationType,
    ChangeSubscriptionType,
    RemoveObjectType,
    AddObjectType,
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
