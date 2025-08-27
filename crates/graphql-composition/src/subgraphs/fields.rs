use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FieldPath(pub(crate) DefinitionId, pub(crate) StringId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArgumentPath(pub(crate) DefinitionId, pub(crate) StringId, pub(crate) StringId);

/// Fields of objects and interfaces.
#[derive(Default)]
pub(crate) struct Fields {
    /// Output field arguments.
    field_arguments: BTreeMap<ArgumentPath, ArgumentId>,

    field_argument_defaults: HashMap<ArgumentPath, Value>,
    input_field_default_values: HashMap<FieldPath, Value>,

    /// Fields of objects, interfaces and input objects.
    ///
    /// FieldIds only become stable once we start composition, since we are sorting at that point. Do not create field ids during ingestion.
    pub(super) fields: Vec<FieldTuple>,
    /// Arguments of output fields.
    pub(super) arguments: Vec<ArgumentRecord>,
}

/// An argument on an output field.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub(crate) struct ArgumentRecord {
    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter!): [User!]!
    ///                ^^^^^^^
    /// }
    /// ```
    pub(crate) name: StringId,
    pub(crate) parent_field: FieldPath,
    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter!): [User!]!
    ///                         ^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) r#type: FieldType,
    pub(crate) description: Option<StringId>,
    pub(crate) directives: DirectiveSiteId,
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
    pub(crate) fn iter_fields(&self) -> impl Iterator<Item = View<'_, FieldId, FieldTuple>> {
        self.fields.fields.iter().enumerate().map(|(index, record)| View {
            id: index.into(),
            record,
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
        self.fields.fields.push(FieldTuple {
            name,
            parent_definition_id,
            r#type: field_type,
            directives,
            description,
        });

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
        let argument_id = self
            .fields
            .arguments
            .push_return_idx(ArgumentRecord {
                name: argument_name,
                r#type,
                directives,
                description,
                parent_field: FieldPath(definition_id, field_name),
            })
            .into();

        self.fields.field_arguments.insert(argument_path, argument_id);

        if let Some(default) = default {
            self.fields.field_argument_defaults.insert(argument_path, default);
        }
    }

    pub(crate) fn iter_output_field_arguments(
        &self,
    ) -> impl ExactSizeIterator<Item = View<'_, ArgumentId, ArgumentRecord>> {
        self.fields
            .arguments
            .iter()
            .enumerate()
            .map(|(idx, record)| View { id: idx.into(), record })
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
}

impl DefinitionId {
    pub(crate) fn fields(self, subgraphs: &Subgraphs) -> impl Iterator<Item = View<'_, FieldId, FieldTuple>> {
        let start = subgraphs
            .fields
            .fields
            .partition_point(|field| field.parent_definition_id < self);

        subgraphs.fields.fields[start..]
            .iter()
            .take_while(move |field| field.parent_definition_id == self)
            .enumerate()
            .map(move |(idx, field)| View {
                id: (start + idx).into(),
                record: field,
            })
    }

    pub(crate) fn field_by_name(self, subgraphs: &Subgraphs, name: StringId) -> Option<View<'_, FieldId, FieldTuple>> {
        subgraphs
            .fields
            .fields
            .binary_search_by_key(&(self, name), |field| (field.parent_definition_id, field.name))
            .ok()
            .map(|idx| View {
                id: idx.into(),
                record: &subgraphs.fields.fields[idx],
            })
    }
}

impl<'a> DefinitionWalker<'a> {
    pub(crate) fn fields(self) -> impl Iterator<Item = FieldWalker<'a>> + 'a {
        self.id
            .fields(self.subgraphs)
            .map(move |field| self.walk((FieldPath(self.id, field.name), *field.record)))
    }

    pub(crate) fn field_by_name(self, name: StringId) -> Option<FieldWalker<'a>> {
        self.id
            .field_by_name(self.subgraphs, name)
            .map(|field| self.walk((FieldPath(field.parent_definition_id, field.name), *field.record)))
    }
}

pub(crate) type FieldArgumentWalker<'a> = Walker<'a, (ArgumentPath, &'a ArgumentRecord)>;

impl<'a> FieldArgumentWalker<'a> {
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

    pub(crate) fn default(&self) -> Option<&'a Value> {
        self.subgraphs.fields.field_argument_defaults.get(&self.id.0)
    }
}
