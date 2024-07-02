use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FieldId(pub(super) DefinitionId, pub(super) StringId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArgumentId(DefinitionId, StringId, StringId);

/// Fields of objects and interfaces.
#[derive(Default)]
pub(crate) struct Fields {
    /// Output field arguments.
    field_arguments: BTreeMap<ArgumentId, FieldTuple>,

    field_argument_defaults: HashMap<ArgumentId, Value>,
    input_field_default_values: HashMap<FieldId, Value>,

    /// Fields of objects, interfaces and input objects.
    definition_fields: BTreeMap<FieldId, FieldTuple>,
}

/// A field in an object, interface or input object type.
#[derive(Clone, Copy)]
pub(crate) struct FieldTuple {
    r#type: FieldTypeId,
    description: Option<StringId>,
    directives: DirectiveSiteId,
}

impl Subgraphs {
    pub(crate) fn iter_all_fields(&self) -> impl Iterator<Item = FieldWalker<'_>> + '_ {
        self.fields
            .definition_fields
            .iter()
            .map(move |(id, tuple)| FieldWalker {
                id: (*id, *tuple),
                subgraphs: self,
            })
    }

    pub(crate) fn push_field(
        &mut self,
        FieldIngest {
            parent_definition_id,
            field_name,
            field_type,
            directives,
            description,
            default,
        }: FieldIngest<'_>,
    ) -> FieldId {
        let name = self.strings.intern(field_name);

        self.fields.definition_fields.insert(
            FieldId(parent_definition_id, name),
            FieldTuple {
                r#type: field_type,
                directives,
                description,
            },
        );

        let field_id = FieldId(parent_definition_id, name);

        if let Some(default) = default {
            self.fields.input_field_default_values.insert(field_id, default);
        }

        field_id
    }

    pub(crate) fn insert_field_argument(
        &mut self,
        FieldId(definition_id, field_name): FieldId,
        argument_name: StringId,
        r#type: FieldTypeId,
        directives: DirectiveSiteId,
        description: Option<StringId>,
        default: Option<Value>,
    ) {
        let argument_id = ArgumentId(definition_id, field_name, argument_name);

        self.fields.field_arguments.insert(
            argument_id,
            FieldTuple {
                r#type,
                directives,
                description,
            },
        );

        if let Some(default) = default {
            self.fields.field_argument_defaults.insert(argument_id, default);
        }
    }

    pub(crate) fn iter_all_field_arguments(&self) -> impl Iterator<Item = FieldArgumentWalker<'_>> + '_ {
        self.fields
            .field_arguments
            .iter()
            .map(|(id, tuple)| FieldArgumentWalker {
                id: (*id, *tuple),
                subgraphs: self,
            })
    }

    pub(crate) fn walk_field(&self, field_id: FieldId) -> FieldWalker<'_> {
        FieldWalker {
            id: (field_id, self.fields.definition_fields[&field_id]),
            subgraphs: self,
        }
    }
}

pub(crate) struct FieldIngest<'a> {
    pub(crate) parent_definition_id: DefinitionId,
    pub(crate) field_name: &'a str,
    pub(crate) field_type: FieldTypeId,
    pub(crate) description: Option<StringId>,
    pub(crate) directives: DirectiveSiteId,
    pub(crate) default: Option<Value>,
}

pub(crate) type FieldWalker<'a> = Walker<'a, (FieldId, FieldTuple)>;

impl<'a> FieldWalker<'a> {
    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter?, searchQuery: String?): [User!]!
    ///                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn arguments(self) -> impl Iterator<Item = FieldArgumentWalker<'a>> {
        let (FieldId(definition_id, field_name), _tuple) = self.id;
        self.subgraphs
            .fields
            .field_arguments
            .range(
                ArgumentId(definition_id, field_name, StringId::MIN)
                    ..ArgumentId(definition_id, field_name, StringId::MAX),
            )
            .map(|(argument_id, tuple)| FieldArgumentWalker {
                id: (*argument_id, *tuple),
                subgraphs: self.subgraphs,
            })
    }

    pub(crate) fn argument_by_name(self, name: StringId) -> Option<FieldArgumentWalker<'a>> {
        let (FieldId(definition_id, field_name), _tuple) = self.id;
        let argument_id = ArgumentId(definition_id, field_name, name);
        self.subgraphs
            .fields
            .field_arguments
            .get(&argument_id)
            .map(|tuple| FieldArgumentWalker {
                id: (argument_id, *tuple),
                subgraphs: self.subgraphs,
            })
    }

    /// For input fields only, the default value.
    pub(crate) fn default_value(self) -> Option<&'a Value> {
        self.subgraphs.fields.input_field_default_values.get(&self.id.0)
    }

    pub(crate) fn description(self) -> Option<StringWalker<'a>> {
        let (_, tuple) = self.id;
        tuple.description.map(|id| self.walk(id))
    }

    pub(crate) fn directives(self) -> DirectiveSiteWalker<'a> {
        let (_, tuple) = self.id;
        self.walk(tuple.directives)
    }

    pub fn parent_definition(self) -> DefinitionWalker<'a> {
        let (FieldId(parent_definition_id, _), _) = self.id;
        self.walk(parent_definition_id)
    }

    /// ```graphql,ignore
    /// id: ID!
    /// ^^
    /// ```
    pub fn name(self) -> StringWalker<'a> {
        let (FieldId(_, name), _) = self.id;
        self.walk(name)
    }

    /// ```ignore,graphql
    /// type MyObject {
    ///   id: ID!
    ///   nested: [Nested!]!
    ///           ^^^^^^^^^^
    /// }
    pub(crate) fn r#type(self) -> FieldTypeWalker<'a> {
        let (_, tuple) = self.id;
        self.walk(tuple.r#type)
    }
}

impl<'a> DefinitionWalker<'a> {
    pub(crate) fn fields(self) -> impl Iterator<Item = FieldWalker<'a>> + 'a {
        self.subgraphs
            .fields
            .definition_fields
            .range(FieldId(self.id, StringId::MIN)..FieldId(self.id, StringId::MAX))
            .map(|(id, tuple)| FieldWalker {
                id: (*id, *tuple),
                subgraphs: self.subgraphs,
            })
    }

    pub(crate) fn find_field(self, name: StringId) -> Option<FieldWalker<'a>> {
        let field_id = FieldId(self.id, name);

        self.subgraphs
            .fields
            .definition_fields
            .get(&field_id)
            .map(|tuple| FieldWalker {
                id: (field_id, *tuple),
                subgraphs: self.subgraphs,
            })
    }
}

pub(crate) type FieldArgumentWalker<'a> = Walker<'a, (ArgumentId, FieldTuple)>;

impl<'a> FieldArgumentWalker<'a> {
    pub(crate) fn field(&self) -> FieldWalker<'a> {
        let (ArgumentId(definition_id, field_name, _), _) = self.id;
        self.subgraphs.walk_field(FieldId(definition_id, field_name))
    }

    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter?): [User!]!
    ///                ^^^^^^^
    /// }
    /// ```
    pub(crate) fn name(&self) -> StringWalker<'a> {
        let (ArgumentId(_, _, name), _) = self.id;
        self.walk(name)
    }

    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter?): [User!]!
    ///                         ^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn r#type(&self) -> FieldTypeWalker<'a> {
        let (_, tuple) = self.id;
        self.walk(tuple.r#type)
    }

    pub(crate) fn directives(&self) -> DirectiveSiteWalker<'a> {
        let (_, tuple) = self.id;
        self.walk(tuple.directives)
    }

    pub(crate) fn default(&self) -> Option<&'a Value> {
        self.subgraphs.fields.field_argument_defaults.get(&self.id.0)
    }
}
