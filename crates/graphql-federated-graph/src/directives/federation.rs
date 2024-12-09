use crate::{InterfaceId, ObjectId, OverrideLabel, OverrideSource, SelectionSet, SubgraphId, Type};

///```ignore,graphql
/// directive @join__type(
///     graph: join__Graph!,
///     key: join__FieldSet,
///     extension: Boolean! = false,
///     resolvable: Boolean! = true,
///     isInterfaceObject: Boolean! = false
/// ) repeatable on OBJECT | INTERFACE | UNION | ENUM | INPUT_OBJECT | SCALAR
///```
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct JoinTypeDirective {
    pub subgraph_id: SubgraphId,
    pub key: Option<SelectionSet>,
    pub resolvable: bool,
    pub is_interface_object: bool,
}

///```ignore,graphql
/// directive @join__field(
///     graph: join__Graph,
///     requires: join__FieldSet,
///     provides: join__FieldSet,
///     type: String,
///     external: Boolean,
///     override: String,
///     overrideLabel: String
/// ) repeatable on FIELD_DEFINITION | INPUT_FIELD_DEFINITION
/// ```
#[derive(Default, Clone, PartialEq, PartialOrd, Debug)]
pub struct JoinFieldDirective {
    pub subgraph_id: Option<SubgraphId>,
    pub requires: Option<SelectionSet>,
    pub provides: Option<SelectionSet>,
    pub r#type: Option<Type>,
    pub r#override: Option<OverrideSource>,
    pub override_label: Option<OverrideLabel>,
}

///```ignore,graphql
/// directive @join__implements(
///     graph: join__Graph!,
///     interface: String!
/// ) repeatable on OBJECT | INTERFACE
/// ```
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct JoinImplementsDirective {
    pub subgraph_id: SubgraphId,
    pub interface_id: InterfaceId,
}

///```ignore,graphql
/// directive @join__unionMember(
///     graph: join__Graph!,
///     member: String!
/// ) repeatable on UNION
///```
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct JoinUnionMemberDirective {
    pub subgraph_id: SubgraphId,
    pub object_id: ObjectId,
}

///```ignore,graphql
/// directive @join__enumValue(
///     graph: join__Graph!
/// ) repeatable on ENUM_VALUE
///```
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct JoinEnumValueDirective {
    pub subgraph_id: SubgraphId,
}
