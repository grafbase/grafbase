use std::borrow::Cow;
use std::collections::BTreeMap;

use engine_parser::types::Directive;
use engine_parser::Positioned;
use schema::{CompositeTypeId, DefinitionId, FieldDefinitionId, ObjectDefinitionId, TypeRecord};

use super::{BindError, BindResult, Binder};
use crate::operation::bind::coercion::coerce_query_value;
use crate::operation::{QueryModifierRule, SkipIncludeDirective};
use crate::{
    operation::{BoundFieldId, BoundSelectionSet, BoundSelectionSetId, Location, QueryPosition},
    response::SafeResponseKey,
};

impl<'schema, 'p> Binder<'schema, 'p> {
    pub(super) fn bind_merged_selection_sets(
        &mut self,
        ty: CompositeTypeId,
        merged_selection_sets: &[&'p Positioned<engine_parser::types::SelectionSet>],
    ) -> BindResult<BoundSelectionSetId> {
        SelectionSetBinder::new(self).bind(ty, merged_selection_sets)
    }
}

pub(super) struct SelectionSetBinder<'schema, 'parsed, 'binder> {
    binder: &'binder mut Binder<'schema, 'parsed>,
    next_query_position: usize,
    rules_stack: Vec<QueryModifierRule>,
    #[allow(clippy::type_complexity)]
    data_fields: BTreeMap<DataFieldUniqueKey, DataField<'parsed>>,
    #[allow(clippy::type_complexity)]
    typename_fields_by_key_then_by_type_condition:
        BTreeMap<SafeResponseKey, BTreeMap<CompositeTypeId, TypenameField<'parsed>>>,
}

impl<'s, 'p, 'b> std::ops::Deref for SelectionSetBinder<'s, 'p, 'b> {
    type Target = Binder<'s, 'p>;

    fn deref(&self) -> &Self::Target {
        self.binder
    }
}

impl<'s, 'p, 'b> std::ops::DerefMut for SelectionSetBinder<'s, 'p, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.binder
    }
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
struct DataFieldUniqueKey {
    response_key: SafeResponseKey,
    definition_id: FieldDefinitionId,
    rules: Vec<QueryModifierRule>,
}

#[derive(Clone)]
struct DataField<'parsed> {
    query_position: QueryPosition,
    fields: Vec<&'parsed Positioned<engine_parser::types::Field>>,
}

#[derive(Clone)]
struct TypenameField<'parsed> {
    query_position: QueryPosition,
    field: &'parsed Positioned<engine_parser::types::Field>,
}

impl<'schema, 'p, 'binder> SelectionSetBinder<'schema, 'p, 'binder> {
    fn new(binder: &'binder mut Binder<'schema, 'p>) -> Self {
        Self {
            binder,
            next_query_position: 0,
            rules_stack: Vec::new(),
            data_fields: BTreeMap::new(),
            typename_fields_by_key_then_by_type_condition: BTreeMap::new(),
        }
    }

    fn bind(
        mut self,
        ty: CompositeTypeId,
        merged_selection_sets: &[&'p Positioned<engine_parser::types::SelectionSet>],
    ) -> BindResult<BoundSelectionSetId> {
        for selection_set in merged_selection_sets {
            self.register_selection_set_fields(ty, selection_set)?;
        }

        let selection_set = BoundSelectionSet {
            field_ids: self.generate_fields(ty)?,
        };
        self.selection_sets.push(selection_set);

        Ok(BoundSelectionSetId::from(self.selection_sets.len() - 1))
    }

    fn generate_fields(&mut self, ty: CompositeTypeId) -> BindResult<Vec<BoundFieldId>> {
        let mut field_ids = Vec::with_capacity(self.data_fields.len());

        for (
            DataFieldUniqueKey {
                response_key,
                definition_id,
                rules,
            },
            DataField { query_position, fields },
        ) in std::mem::take(&mut self.data_fields)
        {
            let field: &'p Positioned<engine_parser::types::Field> = fields
                .iter()
                .min_by_key(|field| field.pos.line)
                .expect("At least one occurrence");
            let key = response_key.with_position(query_position);
            let selection_set_id =
                CompositeTypeId::maybe_from(self.schema.walk(definition_id).ty().as_ref().definition_id)
                    .map(|ty| {
                        let merged_selection_sets = fields
                            .into_iter()
                            .map(|field| &field.node.selection_set)
                            .collect::<Vec<_>>();
                        self.binder.bind_merged_selection_sets(ty, &merged_selection_sets)
                    })
                    .transpose()?;

            field_ids.push(self.bind_field(key, definition_id, field, selection_set_id, rules)?)
        }

        for (response_key, typename_fields) in std::mem::take(&mut self.typename_fields_by_key_then_by_type_condition) {
            // If there is a __typename field applied for all entities within the selection set, we
            // only keep that one.
            if typename_fields
                .get(&ty)
                .map(|field| {
                    Some(field.query_position) == typename_fields.values().map(|field| field.query_position).min()
                })
                .unwrap_or_default()
            {
                let TypenameField { query_position, field } = typename_fields.get(&ty).unwrap();
                let key = response_key.with_position(*query_position);
                field_ids.push(self.bind_typename_field(ty, key, field)?);

                continue;
            }
            for (type_condition, TypenameField { query_position, field }) in typename_fields {
                let key = response_key.with_position(query_position);
                field_ids.push(self.bind_typename_field(type_condition, key, field)?)
            }
        }

        Ok(field_ids)
    }

    fn register_selection_set_fields(
        &mut self,
        ty: CompositeTypeId,
        selection_set: &'p Positioned<engine_parser::types::SelectionSet>,
    ) -> BindResult<()> {
        let Positioned {
            node: selection_set, ..
        } = selection_set;

        for Positioned { node: selection, .. } in &selection_set.items {
            match selection {
                engine_parser::types::Selection::Field(field) => {
                    self.register_field(ty, field)?;
                }
                engine_parser::types::Selection::FragmentSpread(spread) => {
                    self.register_fragment_spread_fields(ty, spread)?;
                }
                engine_parser::types::Selection::InlineFragment(fragment) => {
                    self.register_inline_fragment_fields(ty, fragment)?;
                }
            }
        }

        Ok(())
    }

    fn register_field(
        &mut self,
        parent: CompositeTypeId,
        field: &'p Positioned<engine_parser::types::Field>,
    ) -> BindResult<()> {
        let name_location: Location = field.pos.try_into()?;
        let name = field.name.node.as_str();
        let response_key = self.response_keys.get_or_intern(
            field
                .alias
                .as_ref()
                .map(|Positioned { node, .. }| node.as_str())
                .unwrap_or_else(|| name),
        );
        let query_position = self.next_query_position(name_location)?;

        if name == "__typename" {
            self.typename_fields_by_key_then_by_type_condition
                .entry(response_key)
                .or_default()
                .entry(parent)
                .or_insert(TypenameField { query_position, field });
            return Ok(());
        }

        let definition_id = match parent {
            CompositeTypeId::Object(object_id) => self.schema.object_field_by_name(object_id, name),
            CompositeTypeId::Interface(interface_id) => self.schema.interface_field_by_name(interface_id, name),
            CompositeTypeId::Union(union_id) => {
                return Err(BindError::UnionHaveNoFields {
                    name: name.to_string(),
                    ty: self.schema.walk(union_id).name().to_string(),
                    location: name_location,
                });
            }
        }
        .ok_or_else(|| BindError::UnknownField {
            container: self.schema.walk(DefinitionId::from(parent)).name().to_string(),
            name: name.to_string(),
            location: name_location,
        })?;

        let n = self.rules_stack.len();
        self.push_new_rules(&field.directives)?;

        let entry = self
            .data_fields
            .entry(DataFieldUniqueKey {
                response_key,
                definition_id,
                rules: {
                    let mut rules = self.rules_stack.clone();
                    rules.sort_unstable();
                    rules
                },
            })
            .or_insert(DataField {
                query_position,
                fields: Vec::new(),
            });

        entry.fields.push(field);

        self.rules_stack.truncate(n);

        Ok(())
    }

    fn register_fragment_spread_fields(
        &mut self,
        parent: CompositeTypeId,
        Positioned { pos, node: spread }: &'p Positioned<engine_parser::types::FragmentSpread>,
    ) -> BindResult<()> {
        let location = (*pos).try_into()?;
        // We always create a new selection set from a named fragment. It may not be split in the
        // same way and we need to validate the type condition each time.
        let name = spread.fragment_name.node.as_str();
        let fragment = self
            .parsed_operation
            .get_fragment(name)
            .ok_or_else(|| BindError::UnknownFragment {
                name: name.to_string(),
                location,
            })?;
        let ty = self.bind_selection_set_type(parent, &fragment.node.type_condition)?;

        let n = self.rules_stack.len();
        self.push_new_rules(&fragment.directives)?;

        self.register_selection_set_fields(ty, &fragment.node.selection_set)?;

        self.rules_stack.truncate(n);
        Ok(())
    }

    fn register_inline_fragment_fields(
        &mut self,
        parent: CompositeTypeId,
        Positioned { node: fragment, .. }: &'p Positioned<engine_parser::types::InlineFragment>,
    ) -> BindResult<()> {
        let ty = fragment
            .type_condition
            .as_ref()
            .map(|condition| self.bind_selection_set_type(parent, condition))
            .transpose()?
            .unwrap_or(parent);

        let n = self.rules_stack.len();
        self.push_new_rules(&fragment.directives)?;

        self.register_selection_set_fields(ty, &fragment.selection_set)?;

        self.rules_stack.truncate(n);
        Ok(())
    }

    fn push_new_rules(&mut self, directives: &[Positioned<Directive>]) -> BindResult<()> {
        let mut skip_include = Vec::new();
        for directive in directives {
            let directive_name = directive.name.node.as_str();
            if matches!(directive_name, "skip" | "include") {
                let argument = directive.arguments.first().ok_or(BindError::MissingDirectiveArgument {
                    name: directive_name.to_string(),
                    location: directive.pos.try_into()?,
                    directive: directive_name.to_string(),
                })?;
                let boolean_definition_id = self.schema.scalar_definition_by_name("Boolean").expect("must exist");
                let input_value_id = coerce_query_value(
                    self,
                    argument.1.pos.try_into()?,
                    TypeRecord {
                        definition_id: boolean_definition_id,
                        wrapping: schema::Wrapping::new(true),
                    },
                    argument.1.node.clone(),
                )?;

                if directive_name == "skip" {
                    skip_include.push(SkipIncludeDirective::SkipIf(input_value_id));
                } else {
                    skip_include.push(SkipIncludeDirective::IncludeIf(input_value_id));
                };
            }
        }
        if !skip_include.is_empty() {
            self.rules_stack.push(QueryModifierRule::SkipInclude {
                directives: skip_include,
            });
        }
        Ok(())
    }

    fn bind_selection_set_type(
        &self,
        parent: CompositeTypeId,
        Positioned { pos, node }: &'p Positioned<engine_parser::types::TypeCondition>,
    ) -> BindResult<CompositeTypeId> {
        let location = (*pos).try_into()?;
        let name = node.on.node.as_str();
        let definition = self
            .schema
            .definition_by_name(name)
            .ok_or_else(|| BindError::UnknownType {
                name: name.to_string(),
                location,
            })?;
        let fragment_ty = match definition {
            DefinitionId::Object(object_id) => CompositeTypeId::Object(object_id),
            DefinitionId::Interface(interface_id) => CompositeTypeId::Interface(interface_id),
            DefinitionId::Union(union_id) => CompositeTypeId::Union(union_id),
            _ => {
                return Err(BindError::InvalidTypeConditionTargetType {
                    name: name.to_string(),
                    location,
                });
            }
        };

        let possible_types = self.get_possible_types(parent);
        let fragment_possible_types = self.get_possible_types(fragment_ty);
        let mut i = 0;
        let mut j = 0;
        while i < possible_types.len() && j < fragment_possible_types.len() {
            match possible_types[i].cmp(&fragment_possible_types[j]) {
                std::cmp::Ordering::Less => i += 1,
                // At least one common object
                std::cmp::Ordering::Equal => return Ok(fragment_ty),
                std::cmp::Ordering::Greater => j += 1,
            }
        }

        Err(BindError::DisjointTypeCondition {
            parent: self.schema.walk(DefinitionId::from(parent)).name().to_string(),
            name: name.to_string(),
            location,
        })
    }

    fn get_possible_types(&self, ty: CompositeTypeId) -> Cow<'schema, [ObjectDefinitionId]> {
        match ty {
            CompositeTypeId::Object(id) => Cow::Owned(vec![id]),
            CompositeTypeId::Interface(id) => Cow::Borrowed(&self.schema[id].possible_type_ids),
            CompositeTypeId::Union(id) => Cow::Borrowed(&self.schema[id].possible_type_ids),
        }
    }

    fn next_query_position(&mut self, location: Location) -> BindResult<QueryPosition> {
        let query_position = self.next_query_position;
        self.next_query_position += 1;
        if query_position == QueryPosition::MAX {
            return Err(BindError::TooManyFields { location });
        }
        Ok(QueryPosition::from(query_position))
    }
}
