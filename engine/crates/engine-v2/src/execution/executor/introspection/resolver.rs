use std::cell::RefCell;

use engine_value::ConstValue;
use schema::{
    introspection::{IntrospectionDataSource, __EnumValue, __Field, __InputValue, __Schema, __Type},
    Definition, DefinitionWalker, EnumValue, FieldWalker, InputValueWalker, Schema, TypeWalker,
};

use crate::{
    execution::walkers::{GroupedFieldSet, ResolvedField},
    response::{ResponseData, ResponseValue},
};

pub struct Resolver<'a> {
    schema: &'a Schema,
    types: &'a IntrospectionDataSource,
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
            // We actually only care about the type definitions.
            types: data_source,
            response_data: RefCell::new(response_data),
        }
    }

    pub fn type_by_name(&mut self, root: ResolvedField<'_>, name: &str) -> ResponseValue {
        let fields = root.collect_fields(self.types.__type.id);
        self.schema
            .definition_by_name(name)
            .map(|definition| self.resolve_type_inner(&fields, self.schema.default_walker().walk(definition)))
            .unwrap_or_default()
    }

    // requiring mut as a sanity check despite the RefCell.
    pub fn schema(&mut self, root: ResolvedField<'_>) -> ResponseValue {
        let walker = self.schema.default_walker();
        let fields = root.collect_fields(self.types.__schema.id);
        let response_object = fields.to_response_object_with(|field| {
            let fields = field.collect_fields(self.types.__type.id);
            match self.types.__schema[field.id()] {
                __Schema::Description => self.schema.description.into(),
                __Schema::Types => walker
                    .definitions()
                    .map(|definition| self.resolve_type_inner(&fields, definition))
                    .collect(),
                __Schema::QueryType => self.resolve_type_inner(&fields, walker.query().into()),
                __Schema::MutationType => walker
                    .mutation()
                    .map(|mutation| self.resolve_type_inner(&fields, mutation.into()))
                    .unwrap_or_default(),
                __Schema::SubscriptionType => walker
                    .subscription()
                    .map(|subscription| self.resolve_type_inner(&fields, subscription.into()))
                    .unwrap_or_default(),
                __Schema::Directives => ResponseValue::List(vec![]),
            }
        });

        ResponseValue::Object(self.response_data.borrow_mut().push_object(response_object))
    }

    fn resolve_type_inner(&self, fields: &GroupedFieldSet<'_>, definition: DefinitionWalker<'_>) -> ResponseValue {
        let response_object = fields.to_response_object_with(|field| match self.types.__type[field.id()] {
            // We should intern them
            __Type::Kind => match definition.id {
                Definition::Scalar(_) => self.types.type_kind.scalar,
                Definition::Object(_) => self.types.type_kind.object,
                Definition::Interface(_) => self.types.type_kind.interface,
                Definition::Union(_) => self.types.type_kind.union,
                Definition::Enum(_) => self.types.type_kind.r#enum,
                Definition::InputObject(_) => self.types.type_kind.input_object,
            }
            .into(),
            __Type::Name => definition.schema_name_id().into(),
            __Type::Description => definition.schema_description_id().into(),
            __Type::Fields => {
                let fields = field.collect_fields(self.types.__field.id);
                definition
                    .fields()
                    .map(|type_fields| {
                        // There is a single argument if any so don't need to match anything, the
                        // query is already validated.
                        let include_deprecated = field
                            .bound_arguments()
                            .next()
                            .map(|arg| match arg.resolved_value() {
                                ConstValue::Boolean(b) => b,
                                _ => panic!("Expected boolean argument"),
                            })
                            .unwrap_or_default();
                        type_fields
                            .filter_map(|type_field| {
                                if (!type_field.is_deprecated || include_deprecated)
                                    && !self.types.meta_fields.contains(&type_field.id)
                                {
                                    Some(self.resolve_field(&fields, type_field))
                                } else {
                                    None
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            }
            __Type::Interfaces => {
                let fields = field.collect_fields(self.types.__type.id);
                definition
                    .interfaces()
                    .map(|interfaces| {
                        interfaces
                            .map(|interface| self.resolve_type_inner(&fields, interface.into()))
                            .collect()
                    })
                    .unwrap_or_default()
            }
            __Type::PossibleTypes => {
                let fields = field.collect_fields(self.types.__type.id);
                definition
                    .possible_types()
                    .map(|possible_types| {
                        possible_types
                            .map(|object| self.resolve_type_inner(&fields, object.into()))
                            .collect()
                    })
                    .unwrap_or_default()
            }
            __Type::EnumValues => definition
                .as_enum()
                .map(|r#enum| {
                    // There is a single argument if any so don't need to match anything, the
                    // query is already validated.
                    let include_deprecated = field
                        .bound_arguments()
                        .next()
                        .map(|arg| match arg.resolved_value() {
                            ConstValue::Boolean(b) => b,
                            _ => panic!("Expected boolean argument"),
                        })
                        .unwrap_or_default();
                    let fields = field.collect_fields(self.types.__enum_value.id);
                    r#enum
                        .values()
                        .filter_map(|value| {
                            if !value.is_deprecated || include_deprecated {
                                Some(self.resolve_enum_value(&fields, value))
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default(),
            __Type::InputFields => definition
                .as_input_object()
                .map(|input_object| {
                    let fields = field.collect_fields(self.types.__input_value.id);
                    input_object
                        .input_fields()
                        .map(|input_value| self.resolve_input_value(&fields, input_value))
                        .collect()
                })
                .unwrap_or_default(),
            // Only relevant for field types with a wrapping
            __Type::OfType => ResponseValue::Null,
            __Type::SpecifiedByURL => definition
                .as_scalar()
                .map(|scalar| scalar.specified_by_url.into())
                .unwrap_or_default(),
        });

        ResponseValue::Object(self.response_data.borrow_mut().push_object(response_object))
    }

    fn resolve_enum_value(&self, fields: &GroupedFieldSet<'_>, value: &EnumValue) -> ResponseValue {
        let response_object = fields.to_response_object_with(|field| match self.types.__enum_value[field.id()] {
            __EnumValue::Name => value.name.into(),
            __EnumValue::Description => value.description.into(),
            __EnumValue::IsDeprecated => value.is_deprecated.into(),
            __EnumValue::DeprecationReason => value.deprecated_reason.into(),
        });

        ResponseValue::Object(self.response_data.borrow_mut().push_object(response_object))
    }

    fn resolve_input_value(&self, fields: &GroupedFieldSet<'_>, walker: InputValueWalker<'_>) -> ResponseValue {
        let input_value = walker.get();
        let response_object = fields.to_response_object_with(|field| match self.types.__input_value[field.id()] {
            __InputValue::Name => input_value.name.into(),
            __InputValue::Description => input_value.description.into(),
            __InputValue::Type => self.resolve_type(&field.collect_fields(self.types.__type.id), walker.ty()),
            // TODO: add default value...
            __InputValue::DefaultValue => ResponseValue::Null,
        });
        ResponseValue::Object(self.response_data.borrow_mut().push_object(response_object))
    }

    fn resolve_field(&self, fields: &GroupedFieldSet<'_>, walker: FieldWalker<'_>) -> ResponseValue {
        let target_field = walker.get();
        let response_object = fields.to_response_object_with(|field| match self.types.__field[field.id()] {
            __Field::Name => target_field.name.into(),
            __Field::Description => target_field.description.into(),
            __Field::Args => {
                let fields = field.collect_fields(self.types.__input_value.id);
                walker
                    .arguments()
                    .map(|input_value| self.resolve_input_value(&fields, input_value))
                    .collect()
            }
            __Field::Type => self.resolve_type(&field.collect_fields(self.types.__type.id), walker.ty()),
            __Field::IsDeprecated => target_field.is_deprecated.into(),
            __Field::DeprecationReason => target_field.deprecated_reason.into(),
        });
        ResponseValue::Object(self.response_data.borrow_mut().push_object(response_object))
    }

    fn resolve_type(&self, fields: &GroupedFieldSet<'_>, r#type: TypeWalker<'_>) -> ResponseValue {
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
        self.recursive_resolve_type(fields, r#type.inner(), wrapping)
    }

    fn recursive_resolve_type(
        &self,
        fields: &GroupedFieldSet<'_>,
        definition: DefinitionWalker<'_>,
        mut wrapping: Wrapping,
    ) -> ResponseValue {
        match wrapping.pop() {
            Some(WrappingType::List) => {
                let response_object = fields.to_response_object_with(|field| match self.types.__type[field.id()] {
                    __Type::Kind => self.types.type_kind.list.into(),
                    __Type::OfType => self.recursive_resolve_type(
                        &field.collect_fields(self.types.__type.id),
                        definition,
                        wrapping.clone(),
                    ),
                    _ => ResponseValue::Null,
                });
                ResponseValue::Object(self.response_data.borrow_mut().push_object(response_object))
            }
            Some(WrappingType::NonNull) => {
                let response_object = fields.to_response_object_with(|field| match self.types.__type[field.id()] {
                    __Type::Kind => self.types.type_kind.non_null.into(),
                    __Type::OfType => self.recursive_resolve_type(
                        &field.collect_fields(self.types.__type.id),
                        definition,
                        wrapping.clone(),
                    ),
                    _ => ResponseValue::Null,
                });
                ResponseValue::Object(self.response_data.borrow_mut().push_object(response_object))
            }
            None => self.resolve_type_inner(fields, definition),
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
