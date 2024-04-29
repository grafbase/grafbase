/// Where a directive can apply to.
///
/// [Reference](https://spec.graphql.org/October2021/#DirectiveLocation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DirectiveLocation {
    /// A [query](enum.OperationType.html#variant.Query) [operation](struct.OperationDefinition.html).
    Query,
    /// A [mutation](enum.OperationType.html#variant.Mutation) [operation](struct.OperationDefinition.html).
    Mutation,
    /// A [subscription](enum.OperationType.html#variant.Subscription) [operation](struct.OperationDefinition.html).
    Subscription,
    /// A [field](struct.Field.html).
    Field,
    /// A [fragment definition](struct.FragmentDefinition.html).
    FragmentDefinition,
    /// A [fragment spread](struct.FragmentSpread.html).
    FragmentSpread,
    /// An [inline fragment](struct.InlineFragment.html).
    InlineFragment,
    /// A [schema](struct.Schema.html).
    Schema,
    /// A [scalar](enum.TypeKind.html#variant.Scalar).
    Scalar,
    /// An [object](struct.ObjectType.html).
    Object,
    /// A [field definition](struct.FieldDefinition.html).
    FieldDefinition,
    /// An [input value definition](struct.InputFieldDefinition.html) as the arguments of a field
    /// but not an input object.
    ArgumentDefinition,
    /// An [interface](struct.InterfaceType.html).
    Interface,
    /// A [union](struct.UnionType.html).
    Union,
    /// An [enum](struct.EnumType.html).
    Enum,
    /// A [value on an enum](struct.EnumValueDefinition.html).
    EnumValue,
    /// An [input object](struct.InputObjectType.html).
    InputObject,
    /// An [input value definition](struct.InputValueDefinition.html) on an input object but not a
    /// field.
    InputFieldDefinition,
    /// An [variable definition](struct.VariableDefinition.html).
    VariableDefinition,
}
