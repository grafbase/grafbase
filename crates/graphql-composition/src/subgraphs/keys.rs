use super::*;
use cynic_parser::ConstValue;
use std::collections::HashSet;

/// All the keys (`@key(...)`) in all the subgraphs in one container.
#[derive(Default)]
pub(crate) struct Keys {
    pub(super) keys: Vec<Key>,
    nested_key_fields: NestedKeyFields,
}

impl Subgraphs {
    pub(crate) fn iter_keys(&self) -> impl ExactSizeIterator<Item = View<'_, KeyId, Key>> {
        self.keys.keys.iter().enumerate().map(|(idx, key)| View {
            id: idx.into(),
            record: key,
        })
    }

    pub(crate) fn push_key(
        &mut self,
        object_id: DefinitionId,
        selection_set_str: &str,
        resolvable: bool,
    ) -> Result<(), String> {
        let selection_set = self.selection_set_from_str(selection_set_str, "key", "fields")?;

        let key = Key {
            definition_id: object_id,
            selection_set,
            resolvable,
        };
        self.keys.keys.push(key);
        Ok(())
    }

    pub(crate) fn selection_set_from_str(
        &mut self,
        fields: &str,
        directive_name: &str,
        argument_name: &str,
    ) -> Result<Vec<Selection>, String> {
        use cynic_parser::executable as ast;
        let fields = format!("{{ {fields} }}");
        let parsed = cynic_parser::parse_executable_document(&fields).map_err(|err| {
            format!("could not parse the `{argument_name}` argument in `@{directive_name}` as a selection set: {err}")
        })?;

        let Some(operation) = parsed.operations().next() else {
            return Err(format!(
                "The `{argument_name}` argument in `@{directive_name}` must be a selection set"
            ));
        };

        fn build_selection_set(
            selections: ast::Iter<'_, ast::Selection<'_>>,
            subgraphs: &mut Subgraphs,
        ) -> Result<Vec<Selection>, String> {
            selections
                .map(|selection| match selection {
                    ast::Selection::Field(item) => {
                        let field = subgraphs.strings.intern(item.name());
                        let arguments = item
                            .arguments()
                            .map(|argument| {
                                let name = subgraphs.strings.intern(argument.name());
                                let value = crate::ast_value_to_subgraph_value(
                                    ConstValue::try_from(argument.value()).map_err(|_| "variables are not allowed")?,
                                    subgraphs,
                                );

                                Ok((name, value))
                            })
                            .collect::<Result<Vec<_>, String>>()?;

                        let subselection = build_selection_set(item.selection_set(), subgraphs)?;
                        Ok(Selection::Field(FieldSelection {
                            field,
                            arguments,
                            subselection,
                            has_directives: item.directives().next().is_some(),
                        }))
                    }
                    ast::Selection::InlineFragment(fragment) => {
                        let subselection = build_selection_set(fragment.selection_set(), subgraphs)?;
                        let on = fragment
                            .type_condition()
                            .ok_or("inline fragments must have a type condition")?;

                        Ok(Selection::InlineFragment {
                            on: subgraphs.strings.intern(on),
                            subselection,
                            has_directives: fragment.directives().next().is_some(),
                        })
                    }
                    _ => Err("fragment spreads are not allowed.".to_owned()),
                })
                .collect()
        }

        build_selection_set(operation.selection_set(), self)
            .map_err(|error| format!("the `{argument_name}` argument in `@{directive_name}` was invalid: {error}"))
    }

    pub(crate) fn with_nested_key_fields<F>(&mut self, handler: F)
    where
        F: FnOnce(&Subgraphs, &mut NestedKeyFields),
    {
        let mut nested_key_fields = std::mem::take(&mut self.keys.nested_key_fields);
        handler(self, &mut nested_key_fields);
        self.keys.nested_key_fields = nested_key_fields;
    }
}

#[derive(Default)]
pub(crate) struct NestedKeyFields {
    /// Fields that are part of a nested key key _on another type/entity_. Example:
    ///
    /// ```graphql,ignore
    /// type Entity @key(fields: "name nested { identifier }") {
    ///   name: String!
    /// }
    ///
    /// type Nested {
    ///   identifier: ID!
    /// }
    ///
    /// ```
    ///
    /// `Nested.identifier` is a nested key field.
    fields: HashSet<FieldPath>,

    /// Objects that are part of keys that are not defined on the object itself.
    ///
    /// Example:
    ///
    /// ```graphql,ignore
    /// type Entity @key(fields: "name nested { identifier }") {
    ///   name: String!
    /// }
    ///
    /// type Nested {
    ///   identifier: ID!
    /// }
    /// ```
    ///
    /// `Nested` is an object with a nested key.
    objects_with_nested_keys: HashSet<DefinitionId>,
}

impl NestedKeyFields {
    pub(crate) fn insert(&mut self, field: &FieldTuple) {
        self.fields.insert(FieldPath(field.parent_definition_id, field.name));
        self.objects_with_nested_keys.insert(field.parent_definition_id);
    }
}

/// Corresponds to an `@key` annotation.
#[derive(Debug, PartialOrd, PartialEq)]
pub(crate) struct Key {
    pub(crate) definition_id: DefinitionId,
    pub(crate) selection_set: Vec<Selection>,
    pub(crate) resolvable: bool,
}

#[derive(PartialEq, PartialOrd, Debug)]
pub(crate) enum Selection {
    Field(FieldSelection),
    InlineFragment {
        on: StringId,
        subselection: Vec<Selection>,
        has_directives: bool,
    },
}

#[derive(PartialEq, PartialOrd, Debug)]
pub(crate) struct FieldSelection {
    pub(crate) field: StringId,
    pub(crate) arguments: Vec<(StringId, Value)>,
    pub(crate) subselection: Vec<Selection>,
    pub(crate) has_directives: bool,
}

pub(crate) type KeyWalker<'a> = Walker<'a, KeyId>;

impl<'a> KeyWalker<'a> {
    pub(crate) fn fields(self) -> &'a [Selection] {
        self.view().record.selection_set.as_slice()
    }

    pub(crate) fn is_resolvable(self) -> bool {
        self.view().resolvable
    }

    pub(crate) fn parent_definition(self) -> DefinitionWalker<'a> {
        self.walk(self.view().definition_id)
    }
}

impl<'a> DefinitionWalker<'a> {
    pub(crate) fn is_entity(self) -> bool {
        self.entity_keys().next().is_some()
    }

    pub(crate) fn entity_keys(self) -> impl Iterator<Item = KeyWalker<'a>> {
        self.id.keys(self.subgraphs).map(move |view| self.walk(view.id))
    }
}

impl DefinitionId {
    pub(crate) fn is_entity(self, subgraphs: &Subgraphs) -> bool {
        self.keys(subgraphs).next().is_some()
    }

    pub(crate) fn keys(self, subgraphs: &Subgraphs) -> impl Iterator<Item = View<'_, KeyId, Key>> + '_ {
        let start = subgraphs.keys.keys.partition_point(|key| key.definition_id < self);

        subgraphs.keys.keys[start..]
            .iter()
            .take_while(move |key| key.definition_id == self)
            .enumerate()
            .map(move |(idx, record)| View {
                id: KeyId::from(start + idx),
                record,
            })
    }
}

impl FieldWalker<'_> {
    /// Returns true iff there is an `@key` directive containing this field, possibly with others
    /// as part of a composite key.
    pub(crate) fn is_part_of_key(self) -> bool {
        let (field_id @ FieldPath(_, field_name), _) = self.id;
        self.parent_definition()
            .entity_keys()
            .flat_map(|key| key.fields().iter())
            .filter_map(|selection| match selection {
                Selection::Field(FieldSelection { field, .. }) => Some(field),
                _ => None,
            })
            .any(|key_field| *key_field == field_name)
            || self.subgraphs.keys.nested_key_fields.fields.contains(&field_id)
    }
}
