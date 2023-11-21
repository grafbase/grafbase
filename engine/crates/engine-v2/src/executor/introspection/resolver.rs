use std::cell::RefCell;

use engine_value::ConstValue;
use schema::{
    introspection::{IntrospectionDataSource, __EnumValue, __Field, __InputValue, __Schema, __Type},
    Definition, DefinitionWalker, EnumValue, FieldId, FieldWalker, InputValueWalker, Schema, TypeWalker,
};

use crate::{
    request::OperationSelectionSetWalker,
    response::{CompactResponseObject, ResponseData, ResponseValue},
};

pub struct Resolver<'a> {
    schema: &'a Schema,
    data_source: &'a IntrospectionDataSource,
    response_data: RefCell<&'a mut ResponseData>,
}

#[allow(clippy::panic)]
impl<'a> Resolver<'a> {
    pub fn new(
        schema: &'a Schema,
        data_source: &'a IntrospectionDataSource,
        response_data: &'a mut ResponseData,
    ) -> Self {
        Self {
            schema,
            data_source,
            response_data: RefCell::new(response_data),
        }
    }

    pub fn resolve_type_by_name(
        &mut self,
        name: &str,
        selection_set: OperationSelectionSetWalker<'_>,
    ) -> ResponseValue {
        let walker = selection_set.new_walker();
        self.schema
            .definition_by_name(name)
            .map(|definition| self.resolve_type_inner(walker.schema.walk(definition), selection_set))
            .unwrap_or_default()
    }

    // requiring mut as a sanity check despite the RefCell.
    pub fn resolve_schema(&mut self, selection_set: OperationSelectionSetWalker<'_>) -> ResponseValue {
        let walker = selection_set.new_walker();
        let fields = selection_set
            .all_fields()
            .map(|field| match self.schema_field(field.id()) {
                __Schema::Description => self.schema.description.into(),
                __Schema::Types => walker
                    .schema
                    .definitions()
                    .map(|definition| self.resolve_type_inner(definition, field.subselection()))
                    .collect(),
                __Schema::QueryType => self.resolve_type_inner(walker.schema.query().into(), field.subselection()),
                __Schema::MutationType => walker
                    .schema
                    .mutation()
                    .map(|mutation| self.resolve_type_inner(mutation.into(), field.subselection()))
                    .unwrap_or_default(),
                __Schema::SubscriptionType => walker
                    .schema
                    .subscription()
                    .map(|subscription| self.resolve_type_inner(subscription.into(), field.subselection()))
                    .unwrap_or_default(),
                __Schema::Directives => ResponseValue::List(vec![]),
            })
            .collect();

        ResponseValue::Object(
            self.response_data
                .borrow_mut()
                .push_compact_object(CompactResponseObject { fields }),
        )
    }

    fn schema_field(&self, field_id: FieldId) -> __Schema {
        self.data_source
            .schema(field_id)
            .expect("Validation failure: Unexpected field")
    }

    fn resolve_type_inner(
        &self,
        definition: DefinitionWalker<'_>,
        selection_set: OperationSelectionSetWalker<'_>,
    ) -> ResponseValue {
        let fields = selection_set
            .all_fields()
            .map(|field| match self.type_field(field.id()) {
                // We should intern them
                __Type::Kind => match definition.id {
                    Definition::Scalar(_) => self.data_source.type_kind.scalar,
                    Definition::Object(_) => self.data_source.type_kind.object,
                    Definition::Interface(_) => self.data_source.type_kind.interface,
                    Definition::Union(_) => self.data_source.type_kind.union,
                    Definition::Enum(_) => self.data_source.type_kind.r#enum,
                    Definition::InputObject(_) => self.data_source.type_kind.input_object,
                }
                .into(),
                __Type::Name => definition.schema_name_id().into(),
                __Type::Description => definition.schema_description_id().into(),
                __Type::Fields => {
                    definition
                        .fields()
                        .map(|type_fields| {
                            // There is a single argument if any so don't need to match anything, the
                            // query is already validated.
                            let include_deprecated = field
                                .arguments()
                                .next()
                                .map(|arg| match arg.resolved_value() {
                                    ConstValue::Boolean(b) => b,
                                    _ => panic!("Expected boolean argument"),
                                })
                                .unwrap_or_default();
                            type_fields
                                .filter_map(|type_field| {
                                    if (!type_field.is_deprecated || include_deprecated)
                                        && !self.data_source.meta_fields.contains(&type_field.id)
                                    {
                                        Some(self.resolve_field(type_field, field.subselection()))
                                    } else {
                                        None
                                    }
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                }
                __Type::Interfaces => definition
                    .interfaces()
                    .map(|interfaces| {
                        interfaces
                            .map(|interface| self.resolve_type_inner(interface.into(), field.subselection()))
                            .collect()
                    })
                    .unwrap_or_default(),
                __Type::PossibleTypes => definition
                    .possible_types()
                    .map(|possible_types| {
                        possible_types
                            .map(|object| self.resolve_type_inner(object.into(), field.subselection()))
                            .collect()
                    })
                    .unwrap_or_default(),
                __Type::EnumValues => {
                    definition
                        .as_enum()
                        .map(|r#enum| {
                            // There is a single argument if any so don't need to match anything, the
                            // query is already validated.
                            let include_deprecated = field
                                .arguments()
                                .next()
                                .map(|arg| match arg.resolved_value() {
                                    ConstValue::Boolean(b) => b,
                                    _ => panic!("Expected boolean argument"),
                                })
                                .unwrap_or_default();
                            r#enum
                                .values()
                                .filter_map(|value| {
                                    if !value.is_deprecated || include_deprecated {
                                        Some(self.resolve_enum_value(value, field.subselection()))
                                    } else {
                                        None
                                    }
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                }
                __Type::InputFields => definition
                    .as_input_object()
                    .map(|input_object| {
                        input_object
                            .input_fields()
                            .map(|input_value| self.resolve_input_value(input_value, field.subselection()))
                            .collect()
                    })
                    .unwrap_or_default(),
                // Only relevant for field types with a wrapping
                __Type::OfType => ResponseValue::Null,
                __Type::SpecifiedByURL => definition
                    .as_scalar()
                    .map(|scalar| scalar.specified_by_url.into())
                    .unwrap_or_default(),
            })
            .collect();

        ResponseValue::Object(
            self.response_data
                .borrow_mut()
                .push_compact_object(CompactResponseObject { fields }),
        )
    }

    fn type_field(&self, field_id: FieldId) -> __Type {
        self.data_source
            .type_(field_id)
            .expect("Validation failure: Unexpected field")
    }

    fn resolve_enum_value(&self, value: &EnumValue, selection_set: OperationSelectionSetWalker<'_>) -> ResponseValue {
        let fields = selection_set
            .all_fields()
            .map(|field| match self.enum_value_field(field.id()) {
                __EnumValue::Name => value.name.into(),
                __EnumValue::Description => value.description.into(),
                __EnumValue::IsDeprecated => value.is_deprecated.into(),
                __EnumValue::DeprecationReason => value.deprecated_reason.into(),
            })
            .collect();

        ResponseValue::Object(
            self.response_data
                .borrow_mut()
                .push_compact_object(CompactResponseObject { fields }),
        )
    }

    fn enum_value_field(&self, field_id: FieldId) -> __EnumValue {
        self.data_source
            .enum_value(field_id)
            .expect("Validation failure: Unexpected field")
    }

    fn resolve_input_value(
        &self,
        walker: InputValueWalker<'_>,
        selection_set: OperationSelectionSetWalker<'_>,
    ) -> ResponseValue {
        let input_value = walker.get();
        let fields = selection_set
            .all_fields()
            .map(|field| match self.input_value_field(field.id()) {
                __InputValue::Name => input_value.name.into(),
                __InputValue::Description => input_value.description.into(),
                __InputValue::Type => self.resolve_type(walker.ty(), field.subselection()),
                // TODO: add default value...
                __InputValue::DefaultValue => ResponseValue::Null,
            })
            .collect();
        ResponseValue::Object(
            self.response_data
                .borrow_mut()
                .push_compact_object(CompactResponseObject { fields }),
        )
    }

    fn input_value_field(&self, field_id: FieldId) -> __InputValue {
        self.data_source
            .input_value(field_id)
            .expect("Validation failure: Unexpected field")
    }

    fn resolve_field(&self, walker: FieldWalker<'_>, selection_set: OperationSelectionSetWalker<'_>) -> ResponseValue {
        let target_field = walker.get();
        let fields = selection_set
            .all_fields()
            .map(|field| match self.field_field(field.id()) {
                __Field::Name => target_field.name.into(),
                __Field::Description => target_field.description.into(),
                __Field::Args => walker
                    .arguments()
                    .map(|input_value| self.resolve_input_value(input_value, field.subselection()))
                    .collect(),
                __Field::Type => self.resolve_type(walker.ty(), field.subselection()),
                __Field::IsDeprecated => target_field.is_deprecated.into(),
                __Field::DeprecationReason => target_field.deprecated_reason.into(),
            })
            .collect();
        ResponseValue::Object(
            self.response_data
                .borrow_mut()
                .push_compact_object(CompactResponseObject { fields }),
        )
    }

    fn field_field(&self, field_id: FieldId) -> __Field {
        self.data_source
            .field(field_id)
            .expect("Validation failure: Unexpected field")
    }

    fn resolve_type(&self, r#type: TypeWalker<'_>, selection_set: OperationSelectionSetWalker<'_>) -> ResponseValue {
        // Building it from outermost to innermost
        let mut wrapping = vec![];
        let mut schema_wrapping = r#type.wrapping.clone();
        while let Some(list_wrapping) = schema_wrapping.list_wrapping.pop() {
            match list_wrapping {
                schema::ListWrapping::RequiredList => wrapping.extend([WrappingType::NonNull, WrappingType::List]),
                schema::ListWrapping::NullableList => wrapping.push(WrappingType::List),
            }
        }
        if schema_wrapping.inner_is_required {
            wrapping.push(WrappingType::NonNull);
        }
        wrapping.reverse();
        self.recursive_resolve_type(r#type.inner(), wrapping, selection_set)
    }

    fn recursive_resolve_type(
        &self,
        definition: DefinitionWalker<'_>,
        mut wrapping: Wrapping,
        selection_set: OperationSelectionSetWalker<'_>,
    ) -> ResponseValue {
        match wrapping.pop() {
            Some(WrappingType::List) => {
                let fields = selection_set
                    .all_fields()
                    .map(|field| match self.type_field(field.id()) {
                        __Type::Kind => self.data_source.type_kind.list.into(),
                        __Type::OfType => {
                            self.recursive_resolve_type(definition, wrapping.clone(), field.subselection())
                        }
                        _ => ResponseValue::Null,
                    })
                    .collect();
                ResponseValue::Object(
                    self.response_data
                        .borrow_mut()
                        .push_compact_object(CompactResponseObject { fields }),
                )
            }
            Some(WrappingType::NonNull) => {
                let fields = selection_set
                    .all_fields()
                    .map(|field| match self.type_field(field.id()) {
                        __Type::Kind => self.data_source.type_kind.non_null.into(),
                        __Type::OfType => {
                            self.recursive_resolve_type(definition, wrapping.clone(), field.subselection())
                        }
                        _ => ResponseValue::Null,
                    })
                    .collect();
                ResponseValue::Object(
                    self.response_data
                        .borrow_mut()
                        .push_compact_object(CompactResponseObject { fields }),
                )
            }
            None => self.resolve_type_inner(definition, selection_set),
        }
    }
}

// Innermort to outermost
type Wrapping = Vec<WrappingType>;

#[derive(Clone, Copy)]
enum WrappingType {
    NonNull,
    List,
}
