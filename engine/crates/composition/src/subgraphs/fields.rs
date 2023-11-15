use super::*;

/// Fields of objects and interfaces.
#[derive(Default)]
pub(crate) struct Fields(Vec<Field>);

/// The unique identifier for a field in an object, interface or input object field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FieldId(usize);

/// A field in an object, interface or input object type.
struct Field {
    parent_definition_id: DefinitionId,
    name: StringId,
    field_type: FieldTypeId,
    arguments: Vec<(StringId, FieldTypeId)>,
    provides: Option<Vec<Selection>>,
    requires: Option<Vec<Selection>>,
    is_shareable: bool,
    is_external: bool,
}

impl Subgraphs {
    pub(crate) fn iter_fields(&self) -> impl Iterator<Item = FieldWalker<'_>> {
        (0..self.fields.0.len()).map(FieldId).map(|id| self.walk(id))
    }

    pub(crate) fn push_field(
        &mut self,
        parent_definition_id: DefinitionId,
        field_name: &str,
        field_type: FieldTypeId,
        is_shareable: bool,
        is_external: bool,
        provides: Option<&str>,
        requires: Option<&str>,
    ) -> Result<FieldId, String> {
        let provides = provides
            .map(|provides| self.selection_set_from_str(provides))
            .transpose()?;
        let requires = requires
            .map(|requires| self.selection_set_from_str(requires))
            .transpose()?;
        let name = self.strings.intern(field_name);

        if let Some(last_field) = self.fields.0.last() {
            assert!(last_field.parent_definition_id <= parent_definition_id); // this should stay sorted
        }

        let field = Field {
            parent_definition_id,
            name,
            field_type,
            is_shareable,
            is_external,
            arguments: Vec::new(),
            provides,
            requires,
        };
        let id = FieldId(self.fields.0.push_return_idx(field));
        let parent_object_name = self.walk(parent_definition_id).name().id;
        self.field_names.insert((parent_object_name, name, id));
        Ok(id)
    }

    pub(crate) fn push_field_argument(&mut self, field: FieldId, argument_name: &str, argument_type: FieldTypeId) {
        let argument_name = self.strings.intern(argument_name);
        let field = &mut self.fields.0[field.0];
        field.arguments.push((argument_name, argument_type));
    }
}

pub(crate) type FieldWalker<'a> = Walker<'a, FieldId>;
pub(crate) type ArgumentWalker<'a> = Walker<'a, (StringId, FieldTypeId)>;

impl<'a> FieldWalker<'a> {
    fn field(self) -> &'a Field {
        &self.subgraphs.fields.0[self.id.0]
    }

    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter?, searchQuery: String?): [User!]!
    ///                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn arguments(self) -> impl Iterator<Item = ArgumentWalker<'a>> {
        self.field().arguments.iter().map(move |id| self.walk(*id))
    }

    /// Returns true iff there is an `@key` directive containing exactly this field (no composite
    /// key).
    pub fn is_key(self) -> bool {
        let field = self.field();
        self.parent_definition().entity_keys().any(|key| {
            let mut key_fields = key.fields().iter();

            let Some(first_field) = key_fields.next() else {
                return false;
            };

            if key_fields.next().is_some() || !first_field.subselection.is_empty() {
                return false;
            }

            first_field.field == field.name
        })
    }

    pub fn is_external(self) -> bool {
        self.field().is_external
    }

    pub fn is_shareable(self) -> bool {
        self.field().is_shareable
    }

    pub fn parent_definition(self) -> DefinitionWalker<'a> {
        self.walk(self.field().parent_definition_id)
    }

    /// ```graphql,ignore
    /// id: ID!
    /// ^^
    /// ```
    pub fn name(self) -> StringWalker<'a> {
        self.walk(self.field().name)
    }

    /// ```ignore,graphql
    /// type MyObject {
    ///   id: ID!
    ///   others: [OtherObject!] @provides("size weight")
    ///                          ^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn provides(self) -> Option<&'a [Selection]> {
        self.field().provides.as_deref()
    }

    /// ```ignore.graphql
    /// extend type Farm @federation__key(fields: "id") {
    ///   id: ID! @federation__external
    ///   chiliId: ID! @federation__external
    ///   chiliDetails: ChiliVariety @federation__requires(fields: "chiliId")
    ///                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn requires(self) -> Option<&'a [Selection]> {
        self.field().requires.as_deref()
    }

    /// ```ignore,graphql
    /// type MyObject {
    ///   id: ID!
    ///   nested: [Nested!]!
    ///           ^^^^^^^^^^
    /// }
    pub(crate) fn r#type(self) -> FieldTypeWalker<'a> {
        self.walk(self.field().field_type)
    }
}

impl<'a> ArgumentWalker<'a> {
    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter?): [User!]!
    ///                ^^^^^^^
    /// }
    /// ```
    pub(crate) fn argument_name(&self) -> StringWalker<'a> {
        self.walk(self.id.0)
    }

    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter?): [User!]!
    ///                         ^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn argument_type(&self) -> FieldTypeWalker<'a> {
        self.walk(self.id.1)
    }
}

impl<'a> DefinitionWalker<'a> {
    pub(crate) fn fields(self) -> impl Iterator<Item = FieldWalker<'a>> + 'a {
        let fields = &self.subgraphs.fields.0;
        let start = fields.partition_point(move |field| field.parent_definition_id < self.id);
        fields[start..]
            .iter()
            .take_while(move |field| field.parent_definition_id == self.id)
            .enumerate()
            .map(move |(idx, _)| self.walk(FieldId(start + idx)))
    }
}
