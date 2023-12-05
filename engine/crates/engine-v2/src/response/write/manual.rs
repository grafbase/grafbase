use std::collections::{BTreeMap, HashMap};

use schema::{DataType, ListWrapping, ObjectId, StringId, Wrapping};

use super::{ExecutorOutput, ResponseObjectId, WriteResult};
use crate::{
    plan::{ExpectedGoupedField, ExpectedGroupedFields, ExpectedSelectionSet, ExpectedType, PlanBoundaryId},
    request::{PlanFieldDefinition, PlanWalker, SelectionSetType},
    response::{
        BoundResponseKey, GraphqlError, ResponseBoundaryItem, ResponseObject, ResponsePath, ResponseValue, WriteError,
    },
};

pub struct ExpectedSelectionSetWriter<'a> {
    pub(super) walker: PlanWalker<'a>,
    pub(super) data: &'a mut ExecutorOutput,
    pub(super) path: &'a ResponsePath,
    pub(super) selection_set: &'a ExpectedSelectionSet,
}

impl<'a> ExpectedSelectionSetWriter<'a> {
    #[allow(clippy::panic)]
    pub fn expect_known_object(self) -> ExpectedObjectFieldsWriter<'a> {
        match self.selection_set {
            ExpectedSelectionSet::Grouped(ExpectedGroupedFields {
                maybe_boundary_id,
                ty: SelectionSetType::Object(object_id),
                fields,
                typename_fields,
            }) => ExpectedObjectFieldsWriter {
                maybe_boundary_id: *maybe_boundary_id,
                walker: self.walker,
                data: self.data,
                path: self.path,
                object_id: *object_id,
                fields,
                typename_fields,
            },
            _ => panic!("Selection set wasn't a known object."),
        }
    }

    pub(super) fn write_fields(
        self,
        object_id: ObjectId,
        f: impl Fn(GroupedFieldWriter<'_>) -> WriteResult<ResponseValue>,
    ) -> WriteResult<BTreeMap<BoundResponseKey, ResponseValue>> {
        match self.selection_set {
            ExpectedSelectionSet::Grouped(ExpectedGroupedFields {
                maybe_boundary_id,
                fields,
                typename_fields,
                ..
            }) => ExpectedObjectFieldsWriter {
                maybe_boundary_id: *maybe_boundary_id,
                walker: self.walker,
                data: self.data,
                path: self.path,
                object_id,
                fields,
                typename_fields,
            }
            .write_fields(f),
            ExpectedSelectionSet::Arbitrary(_arbitrary) => {
                todo!()
            }
        }
    }
}

pub struct ExpectedObjectFieldsWriter<'a> {
    maybe_boundary_id: Option<PlanBoundaryId>,
    walker: PlanWalker<'a>,
    data: &'a mut ExecutorOutput,
    path: &'a ResponsePath,
    object_id: ObjectId,
    fields: &'a Vec<ExpectedGoupedField>,
    typename_fields: &'a Vec<BoundResponseKey>,
}

impl<'a> ExpectedObjectFieldsWriter<'a> {
    fn write_with(
        &mut self,
        f: impl Fn(GroupedFieldWriter<'_>) -> WriteResult<ResponseValue>,
    ) -> WriteResult<ResponseObjectId> {
        let object = ResponseObject {
            object_id: self.object_id,
            fields: self.write_fields(f)?,
        };
        let id = self.data.push_object(object);
        if let Some(boundary_id) = self.maybe_boundary_id {
            self.data[boundary_id].push(ResponseBoundaryItem {
                response_object_id: id,
                response_path: self.path.clone(),
                object_id: self.object_id,
            });
        }
        Ok(id)
    }

    fn write_fields(
        &mut self,
        f: impl Fn(GroupedFieldWriter<'_>) -> WriteResult<ResponseValue>,
    ) -> WriteResult<BTreeMap<BoundResponseKey, ResponseValue>> {
        let typename_fields = self.typename_fields.clone();
        let typename = self.walker.schema()[self.object_id].name;
        self.fields
            .iter()
            .map(move |grouped_field| {
                let expected_field = self
                    .walker
                    .walk(grouped_field.definition_id)
                    .as_field()
                    .expect("meta fields aren't included in fields");
                let wrapping = expected_field.ty().wrapping.clone();
                let key = grouped_field.bound_response_key;
                let writer = GroupedFieldWriter {
                    expected_field,
                    data: self.data,
                    path: self.path.child(key),
                    expected_type: &grouped_field.ty,
                    wrapping,
                };
                f(writer).map(|value| (key, value))
            })
            .chain(typename_fields.into_iter().map(|bound_response_key| {
                Ok((
                    bound_response_key,
                    ResponseValue::StringId {
                        id: typename,
                        nullable: false,
                    },
                ))
            }))
            .collect()
    }
}

pub struct GroupedFieldWriter<'a> {
    pub expected_field: PlanFieldDefinition<'a>,
    data: &'a mut ExecutorOutput,
    path: ResponsePath,
    expected_type: &'a ExpectedType,
    wrapping: Wrapping,
}

impl<'a> GroupedFieldWriter<'a> {
    pub fn write_null(&mut self) -> WriteResult<ResponseValue> {
        if let Some(list_wrapping) = self.wrapping.list_wrapping.last() {
            if matches!(list_wrapping, ListWrapping::RequiredList) {
                return self.err("Expected a list, found null");
            }
        } else if self.wrapping.inner_is_required {
            return self.err(format!("Expected a {}, found null", self.expected_type));
        }
        Ok(ResponseValue::Null)
    }

    pub fn write_boolean(&mut self, value: bool) -> WriteResult<ResponseValue> {
        if !self.wrapping.list_wrapping.is_empty() {
            return self.err("Expected a list, found a Boolean");
        }
        if !matches!(self.expected_type, ExpectedType::Scalar(DataType::Boolean)) {
            return self.err(format!("Expected a {}, found a Boolean", self.expected_type));
        }
        Ok(ResponseValue::Boolean {
            value,
            nullable: self.wrapping.inner_is_required,
        })
    }

    pub fn write_string_id(&mut self, id: StringId) -> WriteResult<ResponseValue> {
        if !self.wrapping.list_wrapping.is_empty() {
            return self.err("Expected a list, found a String");
        }
        if !matches!(self.expected_type, ExpectedType::Scalar(DataType::String)) {
            return self.err(format!("Expected a {}, found a String", self.expected_type));
        }
        Ok(ResponseValue::StringId {
            id,
            nullable: self.wrapping.inner_is_required,
        })
    }

    pub fn write_opt_string_id(&mut self, value: Option<StringId>) -> WriteResult<ResponseValue> {
        match value {
            Some(value) => self.write_string_id(value),
            None => self.write_null(),
        }
    }

    pub fn write_known_object_with(
        &mut self,
        f: impl Fn(GroupedFieldWriter<'_>) -> WriteResult<ResponseValue>,
    ) -> WriteResult<ResponseValue> {
        self.write_object(|selection_set| selection_set.expect_known_object().write_with(&f))
    }

    fn write_object(
        &mut self,
        f: impl Fn(ExpectedSelectionSetWriter<'_>) -> WriteResult<ResponseObjectId>,
    ) -> WriteResult<ResponseValue> {
        if !self.wrapping.list_wrapping.is_empty() {
            return self.err("Expected a list, found a String");
        }
        if let ExpectedType::Object(selection_set) = &self.expected_type {
            let writer = ExpectedSelectionSetWriter {
                walker: self.expected_field.walk_with((), ()),
                data: self.data,
                path: &self.path,
                selection_set,
            };
            match f(writer) {
                Ok(id) => Ok(ResponseValue::Object {
                    id,
                    nullable: self.wrapping.inner_is_required,
                }),
                Err(err) => {
                    if let WriteError::Any(err) = err {
                        let _ = self.err(err.to_string());
                    }
                    if self.wrapping.inner_is_required {
                        Err(WriteError::ErrorPropagation)
                    } else {
                        Ok(ResponseValue::Null)
                    }
                }
            }
        } else {
            self.err(format!("Expected an Object, found a {}", self.expected_type))
        }
    }

    pub fn write_opt_list_with<F, I, T>(&mut self, item: Option<I>, f: F) -> WriteResult<ResponseValue>
    where
        I: IntoIterator<Item = T>,
        F: Fn(GroupedFieldWriter<'_>, T) -> WriteResult<ResponseValue>,
    {
        match item {
            Some(item) => self.write_list_with(item, f),
            None => self.write_null(),
        }
    }

    pub fn write_empty_list(&mut self) -> WriteResult<ResponseValue> {
        self.write_list_with(Vec::<()>::new(), |_field, _item| unreachable!())
    }

    pub fn write_list_with<F, I, T>(&mut self, items: I, f: F) -> WriteResult<ResponseValue>
    where
        I: IntoIterator<Item = T>,
        F: Fn(GroupedFieldWriter<'_>, T) -> WriteResult<ResponseValue>,
    {
        if let Some(list_wrapping) = self.wrapping.list_wrapping.pop() {
            let inner_is_required = self
                .wrapping
                .list_wrapping
                .last()
                .map(|lw| matches!(lw, ListWrapping::RequiredList))
                .unwrap_or(self.wrapping.inner_is_required);
            let mut list = Vec::new();
            for (index, item) in items.into_iter().enumerate() {
                let writer = GroupedFieldWriter {
                    path: self.path.child(index),
                    expected_field: self.expected_field,
                    data: self.data,
                    expected_type: self.expected_type,
                    wrapping: self.wrapping.clone(),
                };
                match f(writer, item) {
                    Ok(value) => list.push(value),
                    Err(err) => {
                        if let WriteError::Any(err) = err {
                            self.data.push_error(GraphqlError {
                                message: err.to_string(),
                                locations: vec![self.expected_field.name_location()],
                                path: Some(self.path.clone()),
                                extensions: HashMap::with_capacity(0),
                            });
                        }
                        if inner_is_required {
                            return Err(WriteError::ErrorPropagation);
                        }
                        list.push(ResponseValue::Null);
                    }
                }
            }
            Ok(ResponseValue::List {
                id: self.data.push_list(&list),
                nullable: matches!(list_wrapping, ListWrapping::NullableList),
            })
        } else {
            self.err(format!("Expected a {}, found a list", self.expected_type))
        }
    }

    fn err(&mut self, message: impl Into<String>) -> WriteResult<ResponseValue> {
        self.data.push_error(GraphqlError {
            message: message.into(),
            locations: vec![self.expected_field.name_location()],
            path: Some(self.path.clone()),
            extensions: HashMap::with_capacity(0),
        });
        Err(WriteError::ErrorPropagation)
    }
}
