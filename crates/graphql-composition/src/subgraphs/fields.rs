use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FieldPath(pub(crate) DefinitionId, pub(crate) StringId);

pub(crate) type FieldView<'a> = View<'a, FieldId, FieldTuple>;
pub(crate) type ArgumentView<'a> = View<'a, ArgumentId, ArgumentRecord>;

/// Fields of objects and interfaces.
#[derive(Default)]
pub(crate) struct Fields {
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
    pub(crate) parent_field_name: StringId,
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

    pub(crate) fn push_field(&mut self, record: FieldTuple) {
        self.fields.fields.push(record);
    }

    pub(crate) fn insert_field_argument(&mut self, record: ArgumentRecord) {
        self.fields.arguments.push(record);
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
    ) -> impl Iterator<Item = ArgumentView<'a>> + 'b {
        let start = subgraphs.fields.arguments.partition_point(|arg| {
            (arg.parent_definition_id, arg.parent_field_name) < (self.parent_definition_id, self.name)
        });

        subgraphs.fields.arguments[start..]
            .iter()
            .take_while(|arg| {
                (self.parent_definition_id, self.name) == (arg.parent_definition_id, arg.parent_field_name)
            })
            .enumerate()
            .map(move |(idx, record)| View {
                id: (start + idx).into(),
                record,
            })
    }

    pub(crate) fn argument_by_name<'a>(&self, subgraphs: &'a Subgraphs, name: StringId) -> Option<ArgumentView<'a>> {
        subgraphs
            .fields
            .arguments
            .binary_search_by_key(&(self.parent_definition_id, self.name, name), |arg| {
                (arg.parent_definition_id, arg.parent_field_name, arg.name)
            })
            .map(move |idx| {
                let id = ArgumentId::from(idx);
                subgraphs.at(id)
            })
            .ok()
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
