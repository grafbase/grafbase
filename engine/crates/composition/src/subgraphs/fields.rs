use super::*;

/// Fields of objects and interfaces.
#[derive(Default)]
pub(crate) struct Fields(Vec<Field>);

/// The unique identifier for a field in an object, interface or input object field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FieldId(usize);

impl FieldId {
    pub const MIN: Self = Self(usize::MIN);
    pub const MAX: Self = Self(usize::MAX);
}

/// A field in an object, interface or input object type.
pub(super) struct Field {
    pub(super) parent_definition_id: DefinitionId,
    pub(super) name: StringId,
    field_type: FieldTypeId,
    arguments: Vec<FieldArgument>,
    provides: Option<Vec<Selection>>,
    requires: Option<Vec<Selection>>,
    overrides: Option<StringId>,
    is_shareable: bool,
    is_external: bool,
    is_inaccessible: bool,
    // @deprecated
    deprecated: Option<Deprecation>,

    // @tag
    tags: Vec<StringId>,

    description: Option<StringId>,
}

/// Corresponds to an `@deprecated` directive.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Deprecation {
    pub(crate) reason: Option<StringId>,
}

impl Subgraphs {
    pub(crate) fn iter_fields(&self) -> impl Iterator<Item = FieldWalker<'_>> {
        (0..self.fields.0.len()).map(FieldId).map(|id| self.walk(id))
    }

    pub(crate) fn push_field(
        &mut self,
        FieldIngest {
            parent_definition_id,
            field_name,
            field_type,
            is_shareable,
            is_external,
            is_inaccessible,
            provides,
            requires,
            deprecated,
            tags,
            overrides,
            description,
        }: FieldIngest<'_>,
    ) -> Result<FieldId, String> {
        let provides = provides
            .map(|provides| self.selection_set_from_str(provides))
            .transpose()?;
        let requires = requires
            .map(|requires| self.selection_set_from_str(requires))
            .transpose()?;
        let tags = tags.into_iter().map(|tag| self.strings.intern(tag)).collect();
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
            is_inaccessible,
            arguments: Vec::new(),
            provides,
            requires,
            deprecated,
            tags,
            overrides,
            description,
        };
        let id = FieldId(self.fields.0.push_return_idx(field));
        let parent_object_name = self.walk(parent_definition_id).name().id;
        self.field_names.insert((parent_object_name, name, id));
        Ok(id)
    }

    pub(crate) fn push_field_argument(
        &mut self,
        field: FieldId,
        argument_name: &str,
        argument_type: FieldTypeId,
        is_inaccessible: bool,
    ) {
        let argument_name = self.strings.intern(argument_name);
        let field = &mut self.fields.0[field.0];
        field.arguments.push(FieldArgument {
            name: argument_name,
            r#type: argument_type,
            is_inaccessible,
        });
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FieldArgument {
    name: StringId,
    r#type: FieldTypeId,
    is_inaccessible: bool,
}

pub(crate) struct FieldIngest<'a> {
    pub(crate) parent_definition_id: DefinitionId,
    pub(crate) field_name: &'a str,
    pub(crate) field_type: FieldTypeId,
    pub(crate) is_shareable: bool,
    pub(crate) is_external: bool,
    pub(crate) is_inaccessible: bool,
    pub(crate) provides: Option<&'a str>,
    pub(crate) requires: Option<&'a str>,
    pub(crate) deprecated: Option<Deprecation>,
    pub(crate) tags: Vec<&'a str>,
    pub(crate) description: Option<StringId>,

    /// The @override(from: ...) directive.
    pub(crate) overrides: Option<StringId>,
}

pub(crate) type FieldWalker<'a> = Walker<'a, FieldId>;
pub(crate) type ArgumentWalker<'a> = Walker<'a, FieldArgument>;

impl<'a> FieldWalker<'a> {
    pub(super) fn field(self) -> &'a Field {
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

    pub(crate) fn description(self) -> Option<StringWalker<'a>> {
        self.field().description.map(|id| self.walk(id))
    }

    /// The contents of the `@deprecated` directive. `None` in the absence of directive,
    /// `Some(None)` when no reason is provided.
    pub(crate) fn deprecated(self) -> Option<Option<StringWalker<'a>>> {
        self.field()
            .deprecated
            .map(|deprecated| deprecated.reason.map(|deprecated| self.walk(deprecated)))
    }

    /// ```graphql,ignore
    /// type Query {
    ///     findManyUser(
    ///       filters: FindManyUserFilter?,
    ///       searchQuery: String?
    ///     ): [User!]! @tag(name: "Taste") @tag(name: "the") @tag(name: "Rainbow")
    ///                 ^^^^^^^^^^^^^^^^^^^ ^^^^^^^^^^^^^^^^^ ^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn tags(self) -> impl Iterator<Item = StringWalker<'a>> {
        self.field().tags.iter().map(move |id| self.walk(*id))
    }

    pub fn is_external(self) -> bool {
        self.field().is_external
    }

    pub fn is_shareable(self) -> bool {
        self.field().is_shareable
    }

    pub fn is_inaccessible(self) -> bool {
        self.field().is_inaccessible
    }

    /// ```graphql,ignore
    /// type Query {
    ///   getRandomMammoth: Mammoth @override(from: "steppe")
    ///                             ^^^^^^^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub fn overrides(self) -> Option<StringWalker<'a>> {
        self.field().overrides.map(|override_| self.walk(override_))
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
        self.walk(self.id.name)
    }

    /// ```graphql,ignore
    /// type Query {
    ///   findManyUser(filters: FindManyUserFilter?): [User!]!
    ///                         ^^^^^^^^^^^^^^^^^^^
    /// }
    /// ```
    pub(crate) fn argument_type(&self) -> FieldTypeWalker<'a> {
        self.walk(self.id.r#type)
    }

    pub(crate) fn is_inaccessible(&self) -> bool {
        self.id.is_inaccessible
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

    pub(crate) fn find_field(self, name: StringId) -> Option<FieldWalker<'a>> {
        self.fields().find(|f| f.name().id == name)
    }
}
