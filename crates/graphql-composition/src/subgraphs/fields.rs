use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FieldPath(pub(crate) DefinitionId, pub(crate) StringId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArgumentPath(DefinitionId, StringId, StringId);

/// Fields of objects and interfaces.
#[derive(Default)]
pub(crate) struct Fields {
    /// Output field arguments.
    field_arguments: BTreeMap<ArgumentPath, ArgumentId>,

    field_argument_defaults: HashMap<ArgumentPath, Value>,
    input_field_default_values: HashMap<FieldPath, Value>,

    /// Fields by definition, then name.
    definition_fields: BTreeMap<FieldPath, FieldId>,

    /// Fields of objects, interfaces and input objects.
    pub(super) fields: Vec<FieldTuple>,
    /// Arguments of output fields.
    pub(super) arguments: Vec<ArgumentRecord>,
}

/// An argument on an output field.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub(crate) struct ArgumentRecord {
    parent_field_id: FieldId,
    r#type: FieldType,
    description: Option<StringId>,
    directives: DirectiveSiteId,
}

/// A field in an object, interface or input object type.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub(crate) struct FieldTuple {
    pub(crate) name: StringId,
    pub(crate) parent_definition_id: DefinitionId,
    pub(crate) r#type: FieldType,
    pub(crate) description: Option<StringId>,
    pub(crate) directives: DirectiveSiteId,
}

impl Subgraphs {
    pub(crate) fn iter_all_fields(&self) -> impl Iterator<Item = FieldWalker<'_>> + '_ {
        self.fields
            .definition_fields
            .iter()
            .map(move |(path, field_id)| FieldWalker {
                id: (*path, self[*field_id]),
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
    ) -> FieldPath {
        let name = self.strings.intern(field_name);
        let field_id = self.fields.fields.len().into();
        self.fields.fields.push(FieldTuple {
            name,
            parent_definition_id,
            r#type: field_type,
            directives,
            description,
        });

        self.fields
            .definition_fields
            .insert(FieldPath(parent_definition_id, name), field_id);

        let field_path = FieldPath(parent_definition_id, name);

        if let Some(default) = default {
            self.fields.input_field_default_values.insert(field_path, default);
        }

        field_path
    }

    pub(crate) fn insert_field_argument(
        &mut self,
        FieldPath(definition_id, field_name): FieldPath,
        argument_name: StringId,
        r#type: FieldType,
        directives: DirectiveSiteId,
        description: Option<StringId>,
        default: Option<Value>,
    ) {
        let argument_path = ArgumentPath(definition_id, field_name, argument_name);
        let parent_field_id = self.fields.definition_fields[&FieldPath(definition_id, field_name)];
        let argument_id = self
            .fields
            .arguments
            .push_return_idx(ArgumentRecord {
                r#type,
                directives,
                description,
                parent_field_id,
            })
            .into();

        self.fields.field_arguments.insert(argument_path, argument_id);

        if let Some(default) = default {
            self.fields.field_argument_defaults.insert(argument_path, default);
        }
    }

    pub(crate) fn iter_all_field_arguments(&self) -> impl Iterator<Item = FieldArgumentWalker<'_>> + '_ {
        self.fields
            .field_arguments
            .iter()
            .map(|(path, argument_id)| FieldArgumentWalker {
                id: (*path, &self[*argument_id]),
                subgraphs: self,
            })
    }

    pub(crate) fn walk_field(&self, field_path: FieldPath) -> FieldWalker<'_> {
        FieldWalker {
            id: (field_path, self[self.fields.definition_fields[&field_path]]),
            subgraphs: self,
        }
    }
}

pub(crate) struct FieldIngest<'a> {
    pub(crate) parent_definition_id: DefinitionId,
    pub(crate) field_name: &'a str,
    pub(crate) field_type: FieldType,
    pub(crate) description: Option<StringId>,
    pub(crate) directives: DirectiveSiteId,
    pub(crate) default: Option<Value>,
}

pub(crate) type FieldWalker<'a> = Walker<'a, (FieldPath, FieldTuple)>;

impl<'a> FieldWalker<'a> {
    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter?, searchQuery: String?): [User!]!
    ///                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn arguments(self) -> impl Iterator<Item = FieldArgumentWalker<'a>> {
        let (FieldPath(definition_id, field_name), _tuple) = self.id;
        self.subgraphs
            .fields
            .field_arguments
            .range(
                ArgumentPath(definition_id, field_name, StringId::MIN)
                    ..ArgumentPath(definition_id, field_name, StringId::MAX),
            )
            .map(|(argument_path, argument_id)| FieldArgumentWalker {
                id: (*argument_path, &self.subgraphs[*argument_id]),
                subgraphs: self.subgraphs,
            })
    }

    pub(crate) fn argument_by_name(self, name: StringId) -> Option<FieldArgumentWalker<'a>> {
        let (FieldPath(definition_id, field_name), _tuple) = self.id;
        let argument_path = ArgumentPath(definition_id, field_name, name);
        self.subgraphs
            .fields
            .field_arguments
            .get(&argument_path)
            .map(|argument_id| FieldArgumentWalker {
                id: (argument_path, &self.subgraphs[*argument_id]),
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
        let (FieldPath(parent_definition_id, _), _) = self.id;
        self.walk(parent_definition_id)
    }

    /// ```graphql,ignore
    /// id: ID!
    /// ^^
    /// ```
    pub fn name(self) -> StringWalker<'a> {
        let (FieldPath(_, name), _) = self.id;
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

    pub(crate) fn is_external(self) -> bool {
        self.directives().external() || self.parent_definition().directives().external()
    }
}

impl<'a> DefinitionWalker<'a> {
    pub(crate) fn fields(self) -> impl Iterator<Item = FieldWalker<'a>> + 'a {
        self.subgraphs
            .fields
            .definition_fields
            .range(FieldPath(self.id, StringId::MIN)..FieldPath(self.id, StringId::MAX))
            .map(|(id, field_id)| FieldWalker {
                id: (*id, self.subgraphs[*field_id]),
                subgraphs: self.subgraphs,
            })
    }

    pub(crate) fn find_field(self, name: StringId) -> Option<FieldWalker<'a>> {
        let field_path = FieldPath(self.id, name);

        self.subgraphs
            .fields
            .definition_fields
            .get(&field_path)
            .map(|field_id| FieldWalker {
                id: (field_path, self.subgraphs[*field_id]),
                subgraphs: self.subgraphs,
            })
    }
}

pub(crate) type FieldArgumentWalker<'a> = Walker<'a, (ArgumentPath, &'a ArgumentRecord)>;

impl<'a> FieldArgumentWalker<'a> {
    pub(crate) fn field(&self) -> FieldWalker<'a> {
        let (ArgumentPath(definition_id, field_name, _), _) = self.id;
        self.subgraphs.walk_field(FieldPath(definition_id, field_name))
    }

    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter?): [User!]!
    ///                ^^^^^^^
    /// }
    /// ```
    pub(crate) fn name(&self) -> StringWalker<'a> {
        let (ArgumentPath(_, _, name), _) = self.id;
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

    pub(crate) fn description(&self) -> Option<StringWalker<'a>> {
        let (_, tuple) = self.id;
        let description = tuple.description?;
        Some(self.walk(description))
    }
}
