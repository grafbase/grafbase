use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use schema::{Definition, FieldId};

use crate::{
    request::{
        BoundFieldId, BoundSelectionSetId, EntityType, FlatField, FlatSelectionSet, FlatSelectionSetId,
        FlatTypeCondition, SelectionSetType,
    },
    response::{BoundResponseKey, ResponseKey},
};

use super::{BoundFieldWalker, OperationWalker};

pub type FlatSelectionSetWalker<'op, 'a, Ty = SelectionSetType> = OperationWalker<'op, Cow<'a, FlatSelectionSet<Ty>>>;
pub type FlatFieldWalker<'op, 'a> = OperationWalker<'op, Cow<'a, FlatField>>;

impl<'op, 'a, Ty: Copy> FlatSelectionSetWalker<'op, 'a, Ty> {
    pub fn id(&self) -> FlatSelectionSetId {
        self.item.id
    }

    pub fn ty(&self) -> Ty {
        self.item.ty
    }

    pub fn fields<'out, 's>(&'s self) -> impl ExactSizeIterator<Item = FlatFieldWalker<'op, 'out>> + 'out
    where
        'a: 'out,
        's: 'out,
    {
        let walker: OperationWalker<'op> = self.walk(());
        self.item
            .fields
            .iter()
            .map(move |flat_field| walker.walk(Cow::Borrowed(flat_field)))
    }

    pub fn group_by_field_id(&self) -> HashMap<FieldId, GroupForFieldId> {
        self.item.fields.iter().fold(HashMap::new(), |mut map, flat_field| {
            let bound_field = self.walk(flat_field.bound_field_id);
            if let Some(field) = bound_field.schema_field() {
                map.entry(field.id())
                    .and_modify(|group| {
                        group.key = bound_field.bound_response_key().min(group.key);
                        group.bound_field_ids.push(bound_field.id())
                    })
                    .or_insert_with(|| GroupForFieldId {
                        key: bound_field.bound_response_key(),
                        bound_field_ids: vec![bound_field.id()],
                    });
            }
            map
        })
    }

    pub fn group_by_response_key(&self) -> HashMap<ResponseKey, GroupForResponseKey> {
        self.item.fields.iter().fold(
            HashMap::<ResponseKey, GroupForResponseKey>::new(),
            |mut groups, flat_field| {
                let field = &self.operation[flat_field.bound_field_id];
                let key = field.bound_response_key();
                let group = groups
                    .entry(field.response_key())
                    .or_insert_with(|| GroupForResponseKey {
                        key,
                        origin_selection_set_ids: HashSet::new(),
                        bound_field_ids: vec![],
                    });
                group.key = group.key.min(key);
                group.bound_field_ids.push(flat_field.bound_field_id);
                group.origin_selection_set_ids.extend(&flat_field.selection_set_path);

                groups
            },
        )
    }

    pub fn into_fields(self) -> impl Iterator<Item = FlatFieldWalker<'op, 'static>> {
        let walker = self.walk(());
        self.item
            .into_owned()
            .fields
            .into_iter()
            .map(move |flat_field| walker.walk(Cow::Owned(flat_field)))
    }

    pub fn partition_fields(
        mut self,
        predicate: impl Fn(FlatFieldWalker<'op, '_>) -> bool,
    ) -> (
        FlatSelectionSetWalker<'op, 'static, Ty>,
        FlatSelectionSetWalker<'op, 'static, Ty>,
    ) {
        let fields = match self.item {
            Cow::Borrowed(selection_set) => selection_set.fields.clone(),
            Cow::Owned(ref mut selection_set) => std::mem::take(&mut selection_set.fields),
        };
        let (left, right) = fields
            .into_iter()
            .partition(|flat_field| predicate(self.walk(Cow::Borrowed(flat_field))));
        (self.with_fields(left), self.with_fields(right))
    }

    fn with_fields(&self, fields: Vec<FlatField>) -> FlatSelectionSetWalker<'op, 'static, Ty> {
        self.walk(Cow::Owned(FlatSelectionSet {
            ty: self.item.ty,
            id: self.item.id,
            fields,
        }))
    }

    pub fn is_empty(&self) -> bool {
        self.item.fields.is_empty()
    }

    pub fn into_inner(self) -> FlatSelectionSet<Ty> {
        self.item.into_owned()
    }
}

impl<'op, 'a> FlatFieldWalker<'op, 'a> {
    pub fn bound_field(&self) -> BoundFieldWalker<'op> {
        self.walk(self.item.bound_field_id)
    }

    pub fn into_item(self) -> FlatField {
        self.item.into_owned()
    }

    pub fn entity_type(&self) -> EntityType {
        match self.operation[*self.selection_set_path.last().unwrap()].ty {
            SelectionSetType::Object(id) => EntityType::Object(id),
            SelectionSetType::Interface(id) => EntityType::Interface(id),
            SelectionSetType::Union(_) => {
                unreachable!("Union have no fields")
            }
        }
    }
}

impl<'op, 'a> std::ops::Deref for FlatFieldWalker<'op, 'a> {
    type Target = FlatField;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

#[derive(Debug)]
pub struct GroupForFieldId {
    pub key: BoundResponseKey,
    pub bound_field_ids: Vec<BoundFieldId>,
}

pub struct GroupForResponseKey {
    pub key: BoundResponseKey,
    pub origin_selection_set_ids: HashSet<BoundSelectionSetId>,
    pub bound_field_ids: Vec<BoundFieldId>,
}

impl<'op, 'a, Ty: Copy + std::fmt::Debug + Into<SelectionSetType>> std::fmt::Debug
    for FlatSelectionSetWalker<'op, 'a, Ty>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = Into::<SelectionSetType>::into(self.ty());
        let ty_name = self.walk_with(ty, Definition::from(ty)).name();

        f.debug_struct("FlatSelectionSet")
            .field("id", &self.id())
            .field("ty", &ty_name)
            .field("fields", &self.fields().collect::<Vec<_>>())
            .finish()
    }
}

impl<'op, 'a> std::fmt::Debug for FlatFieldWalker<'op, 'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("FlatField");
        if let Some(type_condition) = self.item.type_condition.as_ref() {
            fmt.field("type_condition", &self.walk_with(type_condition, ()));
        }
        fmt.field("field", &self.bound_field()).finish()
    }
}

impl<'a> std::fmt::Debug for OperationWalker<'a, &FlatTypeCondition, (), ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.item {
            FlatTypeCondition::Interface(id) => f
                .debug_tuple("Inerface")
                .field(&self.schema_walker.walk(*id).name())
                .finish(),
            FlatTypeCondition::Objects(ids) => f
                .debug_tuple("Objects")
                .field(
                    &ids.iter()
                        .map(|id| self.schema_walker.walk(*id).name())
                        .collect::<Vec<_>>(),
                )
                .finish(),
        }
    }
}
