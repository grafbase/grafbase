use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FieldPath(pub(crate) DefinitionId, pub(crate) StringId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArgumentPath(pub(crate) DefinitionId, pub(crate) StringId, pub(crate) StringId);

pub(crate) type FieldView<'a> = View<'a, FieldId, FieldTuple>;

/// Fields of objects and interfaces.
#[derive(Default)]
pub(crate) struct Fields {
    /// Output field arguments.
    field_arguments: BTreeMap<ArgumentPath, ArgumentId>,

    /// Fields of objects, interfaces and input objects.
    ///
    /// FieldIds only become stable once we start composition, since we are sorting at that point. Do not create field ids during ingestion.
    pub(super) fields: Vec<FieldTuple>,
    /// Arguments of output fields.
    pub(super) arguments: Vec<ArgumentRecord>,
}

/// An argument on an output field.
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub(crate) struct ArgumentRecord {
    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter!): [User!]!
    ///                ^^^^^^^
    /// }
    /// ```
    pub(crate) name: StringId,
    pub(crate) parent_field: FieldPath,
    pub(crate) parent_definition_id: DefinitionId,
    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter!): [User!]!
    ///                         ^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) r#type: FieldType,
    pub(crate) description: Option<StringId>,
    pub(crate) directives: DirectiveSiteId,
    pub(crate) default_value: Option<Value>,
}

/// A field in an object, interface or input object type.
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub(crate) struct FieldTuple {
    pub(crate) name: StringId,
    pub(crate) parent_definition_id: DefinitionId,
    /// ```ignore,graphql
    /// type MyObject {
    ///   id: ID!
    ///   nested: [Nested!]!
    ///           ^^^^^^^^^^
    /// }
    /// ```
    pub(crate) r#type: FieldType,
    pub(crate) description: Option<StringId>,
    pub(crate) directives: DirectiveSiteId,
    pub(crate) input_field_default_value: Option<Value>,
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
            input_field_default_value: default,
        });

        FieldPath(parent_definition_id, name)
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
                parent_definition_id: definition_id,
                name: argument_name,
                r#type,
                directives,
                description,
                parent_field: FieldPath(definition_id, field_name),
                default_value: default,
            })
            .into();

        self.fields.field_arguments.insert(argument_path, argument_id);
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

impl FieldTuple {
    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter?, searchQuery: String?): [User!]!
    ///                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn arguments<'b, 'a: 'b>(
        &'b self,
        subgraphs: &'a Subgraphs,
    ) -> impl Iterator<Item = &'a ArgumentRecord> + 'b {
        subgraphs
            .fields
            .field_arguments
            .range(
                ArgumentPath(self.parent_definition_id, self.name, StringId::MIN)
                    ..ArgumentPath(self.parent_definition_id, self.name, StringId::MAX),
            )
            .map(|(_, argument_id)| &subgraphs[*argument_id])
    }

    pub(crate) fn argument_by_name<'a>(&self, subgraphs: &'a Subgraphs, name: StringId) -> Option<&'a ArgumentRecord> {
        let argument_path = ArgumentPath(self.parent_definition_id, self.name, name);
        subgraphs
            .fields
            .field_arguments
            .get(&argument_path)
            .map(|id| &subgraphs[*id])
    }

    pub(crate) fn is_part_of_key(&self, subgraphs: &Subgraphs) -> bool {
        fn selection_contains_field(field_name: StringId, selection: &Selection) -> bool {
            match selection {
                Selection::Field(field_selection) => field_selection.field == field_name,
                Selection::InlineFragment { subselection, .. } => subselection
                    .iter()
                    .any(|selection| selection_contains_field(field_name, selection)),
            }
        }

        self.parent_definition_id.keys(subgraphs).any(|key| {
            key.fields()
                .iter()
                .any(|field| selection_contains_field(self.name, field))
        })
    }
}

impl DefinitionId {
    pub(crate) fn fields(self, subgraphs: &Subgraphs) -> impl Iterator<Item = FieldView<'_>> {
        let start = subgraphs
            .fields
            .fields
            .partition_point(|field| field.parent_definition_id < self);

        subgraphs.fields.fields[start..]
            .iter()
            .take_while(move |field| field.parent_definition_id == self)
            .enumerate()
            .map(move |(idx, record)| FieldView {
                id: (start + idx).into(),
                record,
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
