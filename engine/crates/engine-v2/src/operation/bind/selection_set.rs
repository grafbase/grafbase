use std::borrow::Cow;

use engine_parser::Positioned;
use im::HashMap;
use schema::{DefinitionId, FieldDefinitionId, ObjectDefinitionId};

use crate::{
    operation::{FieldId, Location, QueryPosition, SelectionSet, SelectionSetId, SelectionSetType},
    response::SafeResponseKey,
};

use super::{BindError, BindResult, Binder};

impl<'schema, 'p> Binder<'schema, 'p> {
    pub(super) fn bind_merged_selection_sets(
        &mut self,
        ty: SelectionSetType,
        merged_selection_sets: &[&'p Positioned<engine_parser::types::SelectionSet>],
    ) -> BindResult<SelectionSetId> {
        SelectionSetBinder::new(self).bind(ty, merged_selection_sets)
    }
}

pub(super) struct SelectionSetBinder<'schema, 'parsed, 'binder> {
    binder: &'binder mut Binder<'schema, 'parsed>,
    next_query_position: usize,
    #[allow(clippy::type_complexity)]
    fields: HashMap<
        (SafeResponseKey, FieldDefinitionId),
        (QueryPosition, Vec<&'parsed Positioned<engine_parser::types::Field>>),
    >,
    #[allow(clippy::type_complexity)]
    typename_fields: HashMap<
        SafeResponseKey,
        HashMap<SelectionSetType, (QueryPosition, &'parsed Positioned<engine_parser::types::Field>)>,
    >,
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

impl<'schema, 'p, 'binder> SelectionSetBinder<'schema, 'p, 'binder> {
    fn new(binder: &'binder mut Binder<'schema, 'p>) -> Self {
        Self {
            binder,
            next_query_position: 0,
            fields: HashMap::new(),
            typename_fields: HashMap::new(),
        }
    }

    fn bind(
        mut self,
        ty: SelectionSetType,
        merged_selection_sets: &[&'p Positioned<engine_parser::types::SelectionSet>],
    ) -> BindResult<SelectionSetId> {
        for selection_set in merged_selection_sets {
            self.register_selection_set_fields(ty, selection_set)?;
        }

        let id = SelectionSetId::from(self.selection_sets.len());
        self.selection_sets.push(SelectionSet::default());

        let mut field_ids = self.generate_fields(ty, id)?;

        field_ids.sort_unstable_by_key(|id| {
            let field = &self[*id];
            (
                field.definition_id().map(|id| self.schema[id].parent_entity_id),
                field.query_position(),
            )
        });

        self.binder[id].field_ids_ordered_by_parent_entity_id_then_position = field_ids;

        Ok(id)
    }

    fn generate_fields(&mut self, ty: SelectionSetType, id: SelectionSetId) -> BindResult<Vec<FieldId>> {
        let mut field_ids = Vec::with_capacity(self.fields.len());

        for ((response_key, definition_id), (query_position, fields)) in std::mem::take(&mut self.fields) {
            let field: &'p Positioned<engine_parser::types::Field> = fields
                .iter()
                .min_by_key(|field| field.pos.line)
                .expect("At least one occurence");
            let bound_response_key = response_key
                .with_position(query_position)
                .ok_or(BindError::TooManyFields {
                    location: field.pos.try_into()?,
                })?;
            let selection_set_id =
                SelectionSetType::maybe_from(self.schema.walk(definition_id).ty().as_ref().definition_id)
                    .map(|ty| {
                        let merged_selection_sets = fields
                            .into_iter()
                            .map(|field| &field.node.selection_set)
                            .collect::<Vec<_>>();
                        self.binder.bind_merged_selection_sets(ty, &merged_selection_sets)
                    })
                    .transpose()?;

            field_ids.push(self.bind_field(id, bound_response_key, definition_id, field, selection_set_id)?)
        }

        for (response_key, typename_fields) in std::mem::take(&mut self.typename_fields) {
            // If there is a __typename field applied for all entities within the selection set, we
            // only keep that one.
            if typename_fields
                .get(&ty)
                .map(|(qpos, _)| Some(qpos) == typename_fields.values().map(|(qpos, _)| qpos).min())
                .unwrap_or_default()
            {
                let (query_position, field) = typename_fields.get(&ty).unwrap();
                let bound_response_key =
                    response_key
                        .with_position(*query_position)
                        .ok_or(BindError::TooManyFields {
                            location: field.pos.try_into()?,
                        })?;
                field_ids.push(self.bind_typename_field(id, ty, bound_response_key, field)?);

                continue;
            }
            for (type_condition, (query_position, field)) in typename_fields {
                let bound_response_key =
                    response_key
                        .with_position(query_position)
                        .ok_or(BindError::TooManyFields {
                            location: field.pos.try_into()?,
                        })?;
                field_ids.push(self.bind_typename_field(id, type_condition, bound_response_key, field)?)
            }
        }

        Ok(field_ids)
    }

    fn register_selection_set_fields(
        &mut self,
        ty: SelectionSetType,
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
        parent: SelectionSetType,
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
        let query_position = self.next_query_position();

        if name == "__typename" {
            self.typename_fields
                .entry(response_key)
                .or_default()
                .entry(parent)
                .or_insert((query_position, field));
            return Ok(());
        }

        let definition_id = match parent {
            SelectionSetType::Object(object_id) => self.schema.object_field_by_name(object_id, name),
            SelectionSetType::Interface(interface_id) => self.schema.interface_field_by_name(interface_id, name),
            SelectionSetType::Union(union_id) => {
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

        self.fields
            .entry((response_key, definition_id))
            .or_insert((query_position, Vec::new()))
            .1
            .push(field);

        Ok(())
    }

    fn register_fragment_spread_fields(
        &mut self,
        parent: SelectionSetType,
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
        self.register_selection_set_fields(ty, &fragment.node.selection_set)?;

        Ok(())
    }

    fn register_inline_fragment_fields(
        &mut self,
        parent: SelectionSetType,
        Positioned { node: fragment, .. }: &'p Positioned<engine_parser::types::InlineFragment>,
    ) -> BindResult<()> {
        let ty = fragment
            .type_condition
            .as_ref()
            .map(|condition| self.bind_selection_set_type(parent, condition))
            .transpose()?
            .unwrap_or(parent);
        self.register_selection_set_fields(ty, &fragment.selection_set)
    }

    fn bind_selection_set_type(
        &self,
        parent: SelectionSetType,
        Positioned { pos, node }: &'p Positioned<engine_parser::types::TypeCondition>,
    ) -> BindResult<SelectionSetType> {
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
            DefinitionId::Object(object_id) => SelectionSetType::Object(object_id),
            DefinitionId::Interface(interface_id) => SelectionSetType::Interface(interface_id),
            DefinitionId::Union(union_id) => SelectionSetType::Union(union_id),
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

    fn get_possible_types(&self, ty: SelectionSetType) -> Cow<'schema, [ObjectDefinitionId]> {
        match ty {
            SelectionSetType::Object(id) => Cow::Owned(vec![id]),
            SelectionSetType::Interface(id) => Cow::Borrowed(&self.schema[id].possible_type_ids),
            SelectionSetType::Union(id) => Cow::Borrowed(&self.schema[id].possible_type_ids),
        }
    }

    fn next_query_position(&mut self) -> QueryPosition {
        let query_position = self.next_query_position;
        self.next_query_position += 1;
        QueryPosition::from(query_position)
    }
}
