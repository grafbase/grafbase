/// A composed federated graph.
///
/// ## API contract
///
/// Guarantees:
///
/// - All the identifiers are correct.
///
/// Does not guarantee:
///
/// - The ordering of items inside each `Vec`.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FederatedGraphV1 {
    pub subgraphs: Vec<Subgraph>,

    pub root_operation_types: RootOperationTypes,
    pub objects: Vec<Object>,
    pub object_fields: Vec<ObjectField>,

    pub interfaces: Vec<Interface>,
    pub interface_fields: Vec<InterfaceField>,

    pub fields: Vec<Field>,

    pub enums: Vec<Enum>,
    pub unions: Vec<Union>,
    pub scalars: Vec<Scalar>,
    pub input_objects: Vec<InputObject>,

    /// All the strings in the supergraph, deduplicated.
    pub strings: Vec<String>,

    /// All the field types in the supergraph, deduplicated.
    pub field_types: Vec<FieldType>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RootOperationTypes {
    pub query: ObjectId,
    pub mutation: Option<ObjectId>,
    pub subscription: Option<ObjectId>,
}

impl std::fmt::Debug for FederatedGraphV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<FederatedGraphV1>()).finish()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Subgraph {
    pub name: StringId,
    pub url: StringId,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Object {
    pub name: StringId,

    pub implements_interfaces: Vec<InterfaceId>,

    #[serde(rename = "resolvable_keys")]
    pub keys: Vec<Key>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,

    #[serde(default)]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ObjectField {
    pub object_id: ObjectId,
    pub field_id: FieldId,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Key {
    /// The subgraph that can resolve the entity with the fields in [Key::fields].
    pub subgraph_id: SubgraphId,

    /// Corresponds to the fields in an `@key` directive.
    pub fields: FieldSet,

    /// Correspond to the `@join__type(isInterfaceObject: true)` directive argument.
    pub is_interface_object: bool,

    #[serde(default = "default_true")]
    pub resolvable: bool,
}

fn default_true() -> bool {
    true
}

pub type FieldSet = Vec<FieldSetItem>;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FieldSetItem {
    pub field: FieldId,
    pub subselection: FieldSet,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Field {
    pub name: StringId,
    pub field_type_id: TypeId,

    /// This is populated only of fields of entities. The Vec includes all subgraphs the field can
    /// be resolved in. For a regular field of an entity, it will be one subgraph, the subgraph
    /// where the entity field is defined. For a shareable field in an entity, this contains the
    /// subgraphs where the shareable field is defined on the entity. It may not be all the
    /// subgraphs.
    ///
    /// On fields of value types and input types, this is empty.
    #[serde(deserialize_with = "deserialize_resolvable_in")]
    pub resolvable_in: Vec<SubgraphId>,

    /// See [FieldProvides].
    pub provides: Vec<FieldProvides>,

    /// See [FieldRequires]
    pub requires: Vec<FieldRequires>,

    /// See [Override].
    pub overrides: Vec<Override>,

    pub arguments: Vec<FieldArgument>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,

    #[serde(default)]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FieldArgument {
    pub name: StringId,
    pub type_id: TypeId,
    #[serde(default)]
    pub composed_directives: Vec<Directive>,
    #[serde(default)]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Directive {
    pub name: StringId,
    pub arguments: Vec<(StringId, Value)>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    String(StringId),
    Int(i64),
    Float(StringId),
    Boolean(bool),
    EnumValue(StringId),
    Object(Vec<(StringId, Value)>),
    List(Vec<Value>),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Definition {
    Scalar(ScalarId),
    Object(ObjectId),
    Interface(InterfaceId),
    Union(UnionId),
    Enum(EnumId),
    InputObject(InputObjectId),
}

#[derive(serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct FieldType {
    pub kind: Definition,

    /// Is the innermost type required?
    ///
    /// Examples:
    ///
    /// - `String` => false
    /// - `String!` => true
    /// - `[String!]` => true
    /// - `[String]!` => false
    pub inner_is_required: bool,

    /// Innermost to outermost.
    pub list_wrappers: Vec<ListWrapper>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ListWrapper {
    RequiredList,
    NullableList,
}

/// Represents an `@provides` directive on a field in a subgraph.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FieldProvides {
    pub subgraph_id: SubgraphId,
    pub fields: FieldSet,
}

/// Represents an `@requires` directive on a field in a subgraph.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FieldRequires {
    pub subgraph_id: SubgraphId,
    pub fields: FieldSet,
}

/// Represents an `@override(graph: .., from: ...)` directive on a field in a subgraph.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Override {
    pub graph: SubgraphId,
    /// Points to a subgraph referenced by name, but this is _not_ validated to allow easier field
    /// migrations between subgraphs.
    pub from: OverrideSource,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum OverrideSource {
    Subgraph(SubgraphId),
    Missing(StringId),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Interface {
    pub name: StringId,

    pub implements_interfaces: Vec<InterfaceId>,

    /// All keys, for entity interfaces.
    #[serde(rename = "resolvable_keys")]
    pub keys: Vec<Key>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,

    #[serde(default)]
    pub description: Option<StringId>,
}

fn deserialize_resolvable_in<'de, D: serde::Deserializer<'de>>(de: D) -> Result<Vec<SubgraphId>, D::Error> {
    use serde::Deserialize;

    struct ResolvableInVisitor;

    impl<'a> serde::de::Visitor<'a> for ResolvableInVisitor {
        type Value = Vec<SubgraphId>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("null, a subgraph id or a list of subgraph ids")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'a>,
        {
            Self::Value::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![])
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![SubgraphId(v as usize)])
        }
    }

    de.deserialize_any(ResolvableInVisitor)
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct InterfaceField {
    pub interface_id: InterfaceId,
    pub field_id: FieldId,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Enum {
    pub name: StringId,
    pub values: Vec<EnumValue>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,

    #[serde(default)]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct EnumValue {
    pub value: StringId,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,

    #[serde(default)]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Union {
    pub name: StringId,
    pub members: Vec<ObjectId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,

    #[serde(default)]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Scalar {
    pub name: StringId,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,

    #[serde(default)]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct InputObject {
    pub name: StringId,
    pub fields: Vec<InputObjectField>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,

    #[serde(default)]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct InputObjectField {
    pub name: StringId,
    pub field_type_id: TypeId,
    pub composed_directives: Vec<Directive>,
    #[serde(default)]
    pub description: Option<StringId>,
}

macro_rules! id_newtypes {
    ($($name:ident + $storage:ident + $out:ident,)*) => {
        $(
            #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
            pub struct $name(pub usize);

            impl From<$name> for usize {
              fn from(value: $name) -> usize {
                value.0
              }
            }

            impl std::ops::Index<$name> for FederatedGraphV1 {
                type Output = $out;

                fn index(&self, index: $name) -> &$out {
                    &self.$storage[index.0]
                }
            }

            impl std::ops::IndexMut<$name> for FederatedGraphV1 {
                fn index_mut(&mut self, index: $name) -> &mut $out {
                    &mut self.$storage[index.0]
                }
            }
        )*
    }
}

id_newtypes! {
    EnumId + enums + Enum,
    FieldId + fields + Field,
    TypeId + field_types + FieldType,
    InputObjectId + input_objects + InputObject,
    InterfaceId + interfaces + Interface,
    ObjectId + objects + Object,
    ScalarId + scalars + Scalar,
    StringId + strings + String,
    SubgraphId + subgraphs + Subgraph,
    UnionId + unions + Union,
}

#[cfg(test)]
mod tests {
    use crate::FederatedGraph;

    #[test]
    fn serde_json_backwards_compatibility() {
        serde_json::from_str::<FederatedGraph>(
            r#"
            {
              "V1": {
                "subgraphs": [
                  {
                    "name": 1,
                    "url": 2
                  },
                  {
                    "name": 3,
                    "url": 4
                  },
                  {
                    "name": 5,
                    "url": 6
                  }
                ],
                "root_operation_types": {
                  "query": 5,
                  "mutation": null,
                  "subscription": 6
                },
                "objects": [
                  {
                    "name": 7,
                    "implements_interfaces": [],
                    "resolvable_keys": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 8,
                    "implements_interfaces": [],
                    "resolvable_keys": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 9,
                    "implements_interfaces": [],
                    "resolvable_keys": [
                      {
                        "subgraph_id": 0,
                        "fields": [
                          {
                            "field": 5,
                            "subselection": []
                          }
                        ],
                        "is_interface_object": false,
                        "resolvable": false
                      },
                      {
                        "subgraph_id": 1,
                        "fields": [
                          {
                            "field": 6,
                            "subselection": []
                          }
                        ],
                        "is_interface_object": false
                      },
                      {
                        "subgraph_id": 1,
                        "fields": [
                          {
                            "field": 5,
                            "subselection": []
                          }
                        ],
                        "is_interface_object": false,
                        "resolvable": true
                      },
                      {
                        "subgraph_id": 2,
                        "fields": [
                          {
                            "field": 6,
                            "subselection": []
                          }
                        ],
                        "is_interface_object": false,
                        "resolvable": true
                      }
                    ],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 10,
                    "implements_interfaces": [],
                    "resolvable_keys": [
                      {
                        "subgraph_id": 0,
                        "fields": [
                          {
                            "field": 9,
                            "subselection": []
                          }
                        ],
                        "is_interface_object": false,
                        "resolvable": true
                      },
                      {
                        "subgraph_id": 2,
                        "fields": [
                          {
                            "field": 9,
                            "subselection": []
                          }
                        ],
                        "is_interface_object": false,
                        "resolvable": true
                      }
                    ],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 11,
                    "implements_interfaces": [],
                    "resolvable_keys": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 12,
                    "implements_interfaces": [],
                    "resolvable_keys": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 13,
                    "implements_interfaces": [],
                    "resolvable_keys": [],
                    "composed_directives": [],
                    "description": null
                  }
                ],
                "object_fields": [
                  {
                    "object_id": 0,
                    "field_id": 0
                  },
                  {
                    "object_id": 1,
                    "field_id": 1
                  },
                  {
                    "object_id": 1,
                    "field_id": 2
                  },
                  {
                    "object_id": 1,
                    "field_id": 3
                  },
                  {
                    "object_id": 1,
                    "field_id": 4
                  },
                  {
                    "object_id": 2,
                    "field_id": 5
                  },
                  {
                    "object_id": 2,
                    "field_id": 6
                  },
                  {
                    "object_id": 2,
                    "field_id": 7
                  },
                  {
                    "object_id": 2,
                    "field_id": 8
                  },
                  {
                    "object_id": 3,
                    "field_id": 9
                  },
                  {
                    "object_id": 3,
                    "field_id": 10
                  },
                  {
                    "object_id": 3,
                    "field_id": 11
                  },
                  {
                    "object_id": 3,
                    "field_id": 12
                  },
                  {
                    "object_id": 3,
                    "field_id": 13
                  },
                  {
                    "object_id": 3,
                    "field_id": 14
                  },
                  {
                    "object_id": 3,
                    "field_id": 15
                  },
                  {
                    "object_id": 3,
                    "field_id": 16
                  },
                  {
                    "object_id": 4,
                    "field_id": 17
                  },
                  {
                    "object_id": 4,
                    "field_id": 18
                  },
                  {
                    "object_id": 4,
                    "field_id": 19
                  },
                  {
                    "object_id": 4,
                    "field_id": 20
                  },
                  {
                    "object_id": 4,
                    "field_id": 21
                  },
                  {
                    "object_id": 5,
                    "field_id": 22
                  },
                  {
                    "object_id": 5,
                    "field_id": 23
                  },
                  {
                    "object_id": 6,
                    "field_id": 24
                  }
                ],
                "interfaces": [],
                "interface_fields": [],
                "fields": [
                  {
                    "name": 3,
                    "field_type_id": 0,
                    "resolvable_in": 0,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 23,
                    "field_type_id": 1,
                    "resolvable_in": null,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 24,
                    "field_type_id": 2,
                    "resolvable_in": null,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 25,
                    "field_type_id": 2,
                    "resolvable_in": null,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 26,
                    "field_type_id": 1,
                    "resolvable_in": 2,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [
                      {
                        "name": 27,
                        "arguments": []
                      }
                    ],
                    "description": null
                  },
                  {
                    "name": 28,
                    "field_type_id": 1,
                    "resolvable_in": null,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 29,
                    "field_type_id": 1,
                    "resolvable_in": null,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 30,
                    "field_type_id": 2,
                    "resolvable_in": null,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 5,
                    "field_type_id": 3,
                    "resolvable_in": 2,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 31,
                    "field_type_id": 4,
                    "resolvable_in": null,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 32,
                    "field_type_id": 1,
                    "resolvable_in": 0,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 33,
                    "field_type_id": 5,
                    "resolvable_in": 0,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 34,
                    "field_type_id": 2,
                    "resolvable_in": null,
                    "provides": [],
                    "requires": [],
                    "overrides": [
                      {
                        "graph": 2,
                        "from": {
                          "Subgraph": 0
                        }
                      }
                    ],
                    "arguments": [],
                    "composed_directives": [],
                    "description": 35
                  },
                  {
                    "name": 36,
                    "field_type_id": 2,
                    "resolvable_in": null,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 37,
                    "field_type_id": 6,
                    "resolvable_in": 0,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 5,
                    "field_type_id": 3,
                    "resolvable_in": 2,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 38,
                    "field_type_id": 7,
                    "resolvable_in": 2,
                    "provides": [],
                    "requires": [
                      {
                        "subgraph_id": 2,
                        "fields": [
                          {
                            "field": 13,
                            "subselection": []
                          }
                        ]
                      }
                    ],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 31,
                    "field_type_id": 4,
                    "resolvable_in": 2,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 39,
                    "field_type_id": 1,
                    "resolvable_in": 2,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 40,
                    "field_type_id": 8,
                    "resolvable_in": 2,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 41,
                    "field_type_id": 9,
                    "resolvable_in": 2,
                    "provides": [
                      {
                        "subgraph_id": 2,
                        "fields": [
                          {
                            "field": 7,
                            "subselection": []
                          }
                        ]
                      }
                    ],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 42,
                    "field_type_id": 10,
                    "resolvable_in": 2,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 43,
                    "field_type_id": 11,
                    "resolvable_in": 0,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 44,
                    "field_type_id": 0,
                    "resolvable_in": 1,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 45,
                    "field_type_id": 9,
                    "resolvable_in": 1,
                    "provides": [],
                    "requires": [],
                    "overrides": [],
                    "arguments": [],
                    "composed_directives": [],
                    "description": null
                  }
                ],
                "enums": [
                  {
                    "name": 14,
                    "values": [
                      {
                        "value": 15,
                        "composed_directives": [],
                        "description": null
                      },
                      {
                        "value": 16,
                        "composed_directives": [],
                        "description": null
                      },
                      {
                        "value": 17,
                        "composed_directives": [],
                        "description": null
                      }
                    ],
                    "composed_directives": [],
                    "description": null
                  }
                ],
                "unions": [],
                "scalars": [
                  {
                    "name": 18,
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 19,
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 20,
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 21,
                    "composed_directives": [],
                    "description": null
                  },
                  {
                    "name": 22,
                    "composed_directives": [],
                    "description": null
                  }
                ],
                "input_objects": [],
                "strings": [
                  "join__Graph",
                  "accounts",
                  "http://127.0.0.1:44677",
                  "products",
                  "http://127.0.0.1:35933",
                  "reviews",
                  "http://127.0.0.1:37741",
                  "Cart",
                  "Picture",
                  "Product",
                  "User",
                  "Review",
                  "Query",
                  "Subscription",
                  "Trustworthiness",
                  "REALLY_TRUSTED",
                  "KINDA_TRUSTED",
                  "NOT_TRUSTED",
                  "String",
                  "ID",
                  "Float",
                  "Boolean",
                  "Int",
                  "url",
                  "width",
                  "height",
                  "altText",
                  "inaccessible",
                  "name",
                  "upc",
                  "price",
                  "id",
                  "username",
                  "profilePicture",
                  "reviewCount",
                  "This used to be part of this subgraph, but is now being overridden from\n`reviews`",
                  "joinedTimestamp",
                  "cart",
                  "trustworthiness",
                  "body",
                  "pictures",
                  "product",
                  "author",
                  "me",
                  "topProducts",
                  "newProducts"
                ],
                "field_types": [
                  {
                    "kind": {
                      "Object": 2
                    },
                    "inner_is_required": true,
                    "list_wrappers": [
                      "RequiredList"
                    ]
                  },
                  {
                    "kind": {
                      "Scalar": 0
                    },
                    "inner_is_required": true,
                    "list_wrappers": []
                  },
                  {
                    "kind": {
                      "Scalar": 4
                    },
                    "inner_is_required": true,
                    "list_wrappers": []
                  },
                  {
                    "kind": {
                      "Object": 4
                    },
                    "inner_is_required": true,
                    "list_wrappers": [
                      "RequiredList"
                    ]
                  },
                  {
                    "kind": {
                      "Scalar": 1
                    },
                    "inner_is_required": true,
                    "list_wrappers": []
                  },
                  {
                    "kind": {
                      "Object": 1
                    },
                    "inner_is_required": false,
                    "list_wrappers": []
                  },
                  {
                    "kind": {
                      "Object": 0
                    },
                    "inner_is_required": true,
                    "list_wrappers": []
                  },
                  {
                    "kind": {
                      "Enum": 0
                    },
                    "inner_is_required": true,
                    "list_wrappers": []
                  },
                  {
                    "kind": {
                      "Object": 1
                    },
                    "inner_is_required": true,
                    "list_wrappers": [
                      "RequiredList"
                    ]
                  },
                  {
                    "kind": {
                      "Object": 2
                    },
                    "inner_is_required": true,
                    "list_wrappers": []
                  },
                  {
                    "kind": {
                      "Object": 3
                    },
                    "inner_is_required": false,
                    "list_wrappers": []
                  },
                  {
                    "kind": {
                      "Object": 3
                    },
                    "inner_is_required": true,
                    "list_wrappers": []
                  }
                ]
              }
            }"#,
        )
        .unwrap();
    }
}
