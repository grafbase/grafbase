use super::{MetaField, MetaInputValue, ObjectType, Registry, UnionType};

/// Builder for [`Registry`].
///
/// The builder API should remain as simple as possible, such that first-time users can understand
/// its purpose and quickly use the builder to generate a [`Registry`] with the relevant data
/// inserted.
///
/// This builder is meant to be extended, including an eventual `from_sdl` method to allow
/// declaritively generating a registry.
///
/// By convention, the following usage is adviced:
///
/// - Initiate the builder using `default()`.
/// - `insert_<type>` methods are quick-fire single-argument functions to add one stubbed variant
///   of _type_. For example, `insert_object` inserts an [`ObjectType`], without any fields, with
///   the given `name` argument.
/// - For more complex use-cases, the `build_<type>` sub-builders can be used. These methods
///   initiate a builder for the given type, allowing you to customize the final instance of said
///   type. For example, `build_object` returns an [`ObjectBuilder`]. See it for more details.
/// - The `finalize_<type>` methods on sub-builders are used to generate the relevant type and
///   return to the root builder (e.g. the [`RegistryBuilder`]).
#[derive(Default)]
pub struct RegistryBuilder {
    registry: Registry,
}

impl RegistryBuilder {
    /// Insert a single [`ObjectType`] into the [`RegistryBuilder`], without any fields.
    pub fn insert_object(mut self, name: impl AsRef<str>) -> Self {
        self.registry.insert_type(ObjectType::new(name.as_ref(), []));
        self
    }

    /// Create a new [`ObjectBuilder`] to add a customized [`ObjectType`] to the current
    /// [`RegistryBuilder`].
    pub fn build_object(self, name: impl ToString) -> ObjectBuilder {
        ObjectBuilder {
            root: self,
            name: name.to_string(),
            object: ObjectType::new("Query", []),
        }
    }

    /// Insert a single [`UnionType`] into the [`RegistryBuilder`], with a list of possible types.
    pub fn insert_union<T: Into<String>>(
        mut self,
        name: impl AsRef<str>,
        members: impl IntoIterator<Item = T>,
    ) -> Self {
        self.registry.insert_type(UnionType::new(name.as_ref(), members));
        self
    }

    /// Finalize the [`RegistryBuilder`], and return the final [`Registry`].
    pub fn finalize(mut self) -> Registry {
        if self.registry.types.contains_key("Query") {
            "Query".clone_into(&mut self.registry.query_type);
        }

        if self.registry.types.contains_key("Mutation") {
            self.registry.mutation_type = Some("Mutation".to_owned());
        }

        self.registry
    }
}

/// Builder to generate an [`ObjectType`].
pub struct ObjectBuilder {
    root: RegistryBuilder,
    name: String,
    object: ObjectType,
}

impl ObjectBuilder {
    /// Insert a single [`MetaField`] into the [`ObjectBuilder`], without any arguments.
    pub fn insert_field(mut self, name: impl ToString, target: impl AsRef<str>) -> Self {
        let name = name.to_string();
        let field = MetaField::new(name.clone(), target.as_ref());

        self.object.fields.insert(name, field);
        self
    }

    /// Create a new [`FieldBuilder`] to add a customized [`MetaField`] to the current
    /// [`ObjectBuilder`].
    pub fn build_field(self, name: impl ToString, target: impl AsRef<str>) -> FieldBuilder {
        let name = name.to_string();
        let field = MetaField::new(name.clone(), target.as_ref());

        FieldBuilder {
            root: self,
            name,
            field,
        }
    }

    /// Finalize the [`ObjectBuilder`] to add a [`MetaField`] to the current [`ObjectBuilder`].
    pub fn finalize_object(self) -> RegistryBuilder {
        let Self { mut root, name, object } = self;

        root.registry.types.insert(name, object.into());
        root
    }
}

/// Builder to generate a [`MetaField`].
pub struct FieldBuilder {
    root: ObjectBuilder,
    name: String,
    field: MetaField,
}

impl FieldBuilder {
    /// Insert a single argument ([`MetaInputValue`]) into the [`FieldBuilder`].
    pub fn insert_argument(mut self, name: impl ToString, kind: impl ToString) -> Self {
        let kind = kind.to_string();
        let value = MetaInputValue::new(kind.clone(), kind);

        self.field.args.insert(name.to_string(), value);
        self
    }

    /// Finalize the [`FieldBuilder`] to add a [`MetaField`] to the root [`ObjectBuilder`].
    pub fn finalize_field(self) -> ObjectBuilder {
        let Self { mut root, name, field } = self;

        root.object.fields.insert(name.clone(), field);
        root
    }
}
