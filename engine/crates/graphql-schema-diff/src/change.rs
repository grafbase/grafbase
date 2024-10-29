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
    /// Contents depend on the change kind:
    ///
    /// - [ChangeQueryType]/[ChangeMutationType]/[ChangeSubscriptionType]: the new type
    /// - [AddObjectType]/[AddUnion]/[AddEnum]/[AddScalar]/[AddInterface]/[AddInputObject]/[AddSchemaDefinition]/[AddDirectiveDefinition]: the whole definition.
    /// - [AddInterfaceImplementation]/[RemoveInterfaceImplementation]: empty
    /// - [ChangeFieldType]/[ChangeFieldArgumentType]: the new type
    /// - [RemoveObjectType]/[RemoveField]/[RemoveUnion]/[RemoveUnionMember]/[RemoveEnum]/[RemoveScalar]/[RemoveInterface]/[RemoveDirectiveDefinition]/[RemoveSchemaDefinition]/[RemoveInputObject]/[RemoveFieldArgument]/[RemoveFieldArgumentDefault]: empty
    /// - [AddField]: the whole field definition
    /// - [AddUnionMember]: the union member added
    /// - [AddEnumValue]/[RemoveEnumValue]: empty
    /// - [AddFieldArgument]: the value of the argument, potentially with the default
    /// - [AddFieldArgumentDefault]/[ChangeFieldArgumentDefault]: the default value of the argument
    pub span: Span,
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
    AddSchemaExtension,
    RemoveSchemaExtension,
    AddInputObject,
    RemoveInputObject,
    AddFieldArgument,
    RemoveFieldArgument,
    AddFieldArgumentDefault,
    RemoveFieldArgumentDefault,
    ChangeFieldArgumentDefault,
    ChangeFieldArgumentType,
}

impl ChangeKind {
    /// The change kind as a string.
    pub fn as_str(&self) -> &'static str {
        use ChangeKind::*;

        match self {
            ChangeQueryType => "ChangeQueryType",
            ChangeMutationType => "ChangeMutationType",
            ChangeSubscriptionType => "ChangeSubscriptionType",
            RemoveObjectType => "RemoveObjectType",
            AddObjectType => "AddObjectType",
            AddInterfaceImplementation => "AddInterfaceImplementation",
            RemoveInterfaceImplementation => "RemoveInterfaceImplementation",
            ChangeFieldType => "ChangeFieldType",
            RemoveField => "RemoveField",
            AddField => "AddField",
            AddUnion => "AddUnion",
            RemoveUnion => "RemoveUnion",
            AddUnionMember => "AddUnionMember",
            RemoveUnionMember => "RemoveUnionMember",
            AddEnum => "AddEnum",
            RemoveEnum => "RemoveEnum",
            AddEnumValue => "AddEnumValue",
            RemoveEnumValue => "RemoveEnumValue",
            AddScalar => "AddScalar",
            RemoveScalar => "RemoveScalar",
            AddInterface => "AddInterface",
            RemoveInterface => "RemoveInterface",
            AddDirectiveDefinition => "AddDirectiveDefinition",
            RemoveDirectiveDefinition => "RemoveDirectiveDefinition",
            AddSchemaDefinition => "AddSchemaDefinition",
            AddSchemaExtension => "AddSchemaExtension",
            RemoveSchemaDefinition => "RemoveSchemaDefinition",
            RemoveSchemaExtension => "RemoveSchemaExtension",
            AddInputObject => "AddInputObject",
            RemoveInputObject => "RemoveInputObject",
            AddFieldArgument => "AddFieldArgument",
            RemoveFieldArgument => "RemoveFieldArgument",
            AddFieldArgumentDefault => "AddFieldArgumentDefault",
            RemoveFieldArgumentDefault => "RemoveFieldArgumentDefault",
            ChangeFieldArgumentDefault => "ChangeFieldArgumentDefault",
            ChangeFieldArgumentType => "ChangeFieldArgumentType",
        }
    }
}

impl std::str::FromStr for ChangeKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "ChangeQueryType" => Self::ChangeQueryType,
            "ChangeMutationType" => Self::ChangeMutationType,
            "ChangeSubscriptionType" => Self::ChangeSubscriptionType,
            "RemoveObjectType" => Self::RemoveObjectType,
            "AddObjectType" => Self::AddObjectType,
            "AddInterfaceImplementation" => Self::AddInterfaceImplementation,
            "RemoveInterfaceImplementation" => Self::RemoveInterfaceImplementation,
            "ChangeFieldType" => Self::ChangeFieldType,
            "RemoveField" => Self::RemoveField,
            "AddField" => Self::AddField,
            "AddUnion" => Self::AddUnion,
            "RemoveUnion" => Self::RemoveUnion,
            "AddUnionMember" => Self::AddUnionMember,
            "RemoveUnionMember" => Self::RemoveUnionMember,
            "AddEnum" => Self::AddEnum,
            "RemoveEnum" => Self::RemoveEnum,
            "AddEnumValue" => Self::AddEnumValue,
            "RemoveEnumValue" => Self::RemoveEnumValue,
            "AddScalar" => Self::AddScalar,
            "RemoveScalar" => Self::RemoveScalar,
            "AddInterface" => Self::AddInterface,
            "RemoveInterface" => Self::RemoveInterface,
            "AddDirectiveDefinition" => Self::AddDirectiveDefinition,
            "RemoveDirectiveDefinition" => Self::RemoveDirectiveDefinition,
            "AddSchemaDefinition" => Self::AddSchemaDefinition,
            "RemoveSchemaDefinition" => Self::RemoveSchemaDefinition,
            "AddSchemaExtension" => Self::AddSchemaExtension,
            "RemoveSchemaExtension" => Self::RemoveSchemaExtension,
            "AddInputObject" => Self::AddInputObject,
            "RemoveInputObject" => Self::RemoveInputObject,
            "AddFieldArgument" => Self::AddFieldArgument,
            "RemoveFieldArgument" => Self::RemoveFieldArgument,
            "AddFieldArgumentDefault" => Self::AddFieldArgumentDefault,
            "RemoveFieldArgumentDefault" => Self::RemoveFieldArgumentDefault,
            "ChangeFieldArgumentDefault" => Self::ChangeFieldArgumentDefault,
            "ChangeFieldArgumentType" => Self::ChangeFieldArgumentType,
            _ => return Err(()),
        })
    }
}

/// A span in a source file.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Debug, serde::Serialize, serde::Deserialize, Copy)]
pub struct Span {
    /// The byte offset where the span starts.
    pub start: usize,
    /// The byte offset where the span stops (exclusive).
    pub end: usize,
}

impl Span {
    /// Create a span from start and end.
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    pub(crate) const fn empty() -> Span {
        Span { start: 0, end: 0 }
    }
}

impl From<cynic_parser::Span> for Span {
    fn from(cynic_parser::Span { start, end }: cynic_parser::Span) -> Self {
        Span { start, end }
    }
}

impl std::ops::Index<Span> for str {
    type Output = str;

    fn index(&self, Span { start, end }: Span) -> &Self::Output {
        &self[start..end]
    }
}
