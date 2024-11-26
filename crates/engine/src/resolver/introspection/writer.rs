use schema::{
    introspection::{
        IntrospectionField, IntrospectionMetadata, IntrospectionObject, _Field, __EnumValue, __InputValue, __Schema,
        __Type,
    },
    Definition, EntityDefinition, EnumValue, FieldDefinition, InputValueDefinition, InterfaceDefinition, ListWrapping,
    ObjectDefinition, Schema, StringId, Type, TypeSystemDirective, Wrapping,
};
use walker::{Iter, Walk};

use crate::{
    execution::ExecutionContext,
    operation::Plan,
    response::{
        ConcreteShapeId, FieldShapeRecord, ResponseObject, ResponseObjectField, ResponseValue, ResponseWriter, Shapes,
    },
    Runtime,
};

pub(super) struct IntrospectionWriter<'ctx, R: Runtime> {
    pub ctx: ExecutionContext<'ctx, R>,
    pub schema: &'ctx Schema,
    pub shapes: &'ctx Shapes,
    pub metadata: &'ctx IntrospectionMetadata,
    pub plan: Plan<'ctx>,
    pub response: ResponseWriter<'ctx>,
}

impl<'ctx, R: Runtime> IntrospectionWriter<'ctx, R> {
    pub(super) fn execute(self, id: ConcreteShapeId) {
        let shape = &self.ctx.shapes()[id];
        let mut fields = Vec::with_capacity(shape.field_shape_ids.len() + shape.typename_response_keys.len());
        for field_shape in &self.shapes[shape.field_shape_ids] {
            let field = field_shape.id.walk(&self.ctx);
            let arguments = field.hydrated_arguments(&self.ctx);
            match self.metadata.root_field(field.definition_id) {
                IntrospectionField::Type => {
                    let name = arguments.get_arg_value_as::<&str>("name");
                    fields.push(ResponseObjectField {
                        key: field_shape.key,
                        required_field_id: None,
                        value: self
                            .schema
                            .definition_by_name(name)
                            .filter(|def| !def.is_inaccessible())
                            .map(|definition| {
                                self.__type_inner(definition, field_shape.shape.as_concrete_object().unwrap())
                            })
                            .into(),
                    });
                }
                IntrospectionField::Schema => {
                    fields.push(ResponseObjectField {
                        key: field_shape.key,
                        required_field_id: None,
                        value: self.__schema(field_shape.shape.as_concrete_object().unwrap()),
                    });
                }
            };
        }
        if !shape.typename_response_keys.is_empty() {
            let name_id = match self.plan.entity_definition() {
                EntityDefinition::Object(object) => object.name_id,
                EntityDefinition::Interface(interface) => interface.name_id,
            };
            for edge in &shape.typename_response_keys {
                fields.push(ResponseObjectField {
                    key: *edge,
                    required_field_id: None,
                    value: name_id.into(),
                });
            }
        }
        self.response.update_root_object_with(fields);
    }

    fn object<E: Copy, const N: usize>(
        &self,
        object: &'ctx IntrospectionObject<E, N>,
        shape_id: ConcreteShapeId,
        build: impl Fn(&'ctx FieldShapeRecord, E) -> ResponseValue,
    ) -> ResponseValue {
        let shape = &self.shapes[shape_id];
        let mut fields = Vec::with_capacity(shape.field_shape_ids.len() + shape.typename_response_keys.len());
        for id in shape.field_shape_ids {
            let field = &self.shapes[id];
            fields.push(ResponseObjectField {
                key: field.key,
                required_field_id: None,
                value: build(field, object[field.id.walk(&self.ctx).definition_id]),
            });
        }
        if !shape.typename_response_keys.is_empty() {
            let name = self.schema.walk(object.id).as_ref().name_id;
            for edge in &shape.typename_response_keys {
                fields.push(ResponseObjectField {
                    key: *edge,
                    required_field_id: None,
                    value: name.into(),
                });
            }
        }

        self.response.push_object(ResponseObject::new(fields)).into()
    }

    fn __schema(&self, shape_id: ConcreteShapeId) -> ResponseValue {
        self.object(&self.metadata.__schema, shape_id, |field, __schema| {
            match __schema {
                __Schema::Description => self.schema.graph.description_id.into(),
                __Schema::Types => {
                    let shape_id = field.shape.as_concrete_object().unwrap();
                    let mut values = self.response.new_list();
                    values.extend(
                        self.schema
                            .definitions()
                            .filter(|def| !def.is_inaccessible())
                            .map(|definition| self.__type_inner(definition, shape_id)),
                    );
                    self.response.push_list(values).into()
                }
                __Schema::QueryType => self.__type_inner(
                    Definition::Object(self.schema.query()),
                    field.shape.as_concrete_object().unwrap(),
                ),
                __Schema::MutationType => self
                    .schema
                    .mutation()
                    .map(|mutation| {
                        self.__type_inner(Definition::Object(mutation), field.shape.as_concrete_object().unwrap())
                    })
                    .unwrap_or_default(),
                __Schema::SubscriptionType => self
                    .schema
                    .subscription()
                    .map(|subscription| {
                        self.__type_inner(
                            Definition::Object(subscription),
                            field.shape.as_concrete_object().unwrap(),
                        )
                    })
                    .unwrap_or_default(),
                // TODO: Need to implemented directives...
                __Schema::Directives => self.response.push_empty_list().into(),
            }
        })
    }

    fn __type(&self, ty: Type<'ctx>, shape_id: ConcreteShapeId) -> ResponseValue {
        self.__type_list_wrapping(ty.definition(), ty.wrapping, shape_id)
    }

    fn __type_list_wrapping(
        &self,
        definition: Definition<'ctx>,
        mut wrapping: Wrapping,
        shape_id: ConcreteShapeId,
    ) -> ResponseValue {
        match wrapping.pop_list_wrapping() {
            Some(list_wrapping) => match list_wrapping {
                ListWrapping::RequiredList => {
                    self.__type_required_wrapping(definition, wrapping.wrapped_by_nullable_list(), shape_id)
                }
                ListWrapping::NullableList => {
                    self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
                        __Type::Kind => self.metadata.type_kind.list.into(),
                        __Type::OfType => {
                            self.__type_list_wrapping(definition, wrapping, field.shape.as_concrete_object().unwrap())
                        }
                        _ => ResponseValue::Null,
                    })
                }
            },
            None => {
                if wrapping.inner_is_required() {
                    self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
                        __Type::Kind => self.metadata.type_kind.non_null.into(),
                        __Type::OfType => self.__type_inner(definition, field.shape.as_concrete_object().unwrap()),
                        _ => ResponseValue::Null,
                    })
                } else {
                    self.__type_inner(definition, shape_id)
                }
            }
        }
    }

    fn __type_required_wrapping(
        &self,
        definition: Definition<'ctx>,
        wrapping: Wrapping,
        shape_id: ConcreteShapeId,
    ) -> ResponseValue {
        self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
            __Type::Kind => self.metadata.type_kind.non_null.into(),
            __Type::OfType => {
                self.__type_list_wrapping(definition, wrapping, field.shape.as_concrete_object().unwrap())
            }
            _ => ResponseValue::Null,
        })
    }

    fn __type_inner(&self, definition: Definition<'ctx>, shape_id: ConcreteShapeId) -> ResponseValue {
        match definition {
            Definition::Scalar(scalar) => self.object(&self.metadata.__type, shape_id, |_, __type| match __type {
                __Type::Kind => self.metadata.type_kind.scalar.into(),
                __Type::Name => scalar.name_id.into(),
                __Type::Description => scalar.description_id.into(),
                __Type::SpecifiedByURL => scalar.specified_by_url_id.into(),
                _ => ResponseValue::Null,
            }),
            Definition::Object(object) => self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
                __Type::Kind => self.metadata.type_kind.object.into(),
                __Type::Name => object.name_id.into(),
                __Type::Description => object.description_id.into(),
                __Type::Fields => self.__type_fields(field, object.fields()),
                __Type::Interfaces => self.__type_interfaces(field, object.interfaces()),
                _ => ResponseValue::Null,
            }),
            Definition::Interface(interface) => {
                self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
                    __Type::Kind => self.metadata.type_kind.interface.into(),
                    __Type::Name => interface.name_id.into(),
                    __Type::Description => interface.description_id.into(),
                    __Type::Fields => self.__type_fields(field, interface.fields()),
                    __Type::Interfaces => self.__type_interfaces(field, interface.interfaces()),
                    __Type::PossibleTypes => self.__type_possible_types(field, interface.possible_types()),
                    _ => ResponseValue::Null,
                })
            }
            Definition::Union(union) => self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
                __Type::Kind => self.metadata.type_kind.union.into(),
                __Type::Name => union.name_id.into(),
                __Type::Description => union.description_id.into(),
                __Type::PossibleTypes => self.__type_possible_types(field, union.possible_types()),
                _ => ResponseValue::Null,
            }),
            Definition::Enum(r#enum) => self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
                __Type::Kind => self.metadata.type_kind.r#enum.into(),
                __Type::Name => r#enum.name_id.into(),
                __Type::Description => r#enum.description_id.into(),
                __Type::EnumValues => {
                    let shape_id = field.shape.as_concrete_object().unwrap();
                    let include_deprecated = field
                        .id
                        .walk(&self.ctx)
                        .hydrated_arguments(&self.ctx)
                        .get_arg_value_as::<bool>("includeDeprecated");
                    let mut values = self.response.new_list();
                    values.extend(
                        r#enum
                            .values()
                            .filter(|value| {
                                !value.is_inaccessible() && (!is_deprecated(value.directives()) || include_deprecated)
                            })
                            .map(|value| self.__enum_value(value, shape_id)),
                    );
                    self.response.push_list(values).into()
                }
                _ => ResponseValue::Null,
            }),
            Definition::InputObject(input_object) => {
                self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
                    __Type::Kind => self.metadata.type_kind.input_object.into(),
                    __Type::Name => input_object.name_id.into(),
                    __Type::Description => input_object.description_id.into(),
                    __Type::InputFields => {
                        let shape_id = field.shape.as_concrete_object().unwrap();
                        let mut values = self.response.new_list();
                        values.extend(
                            input_object
                                .input_fields()
                                .filter(|input_field| !input_field.is_inaccessible())
                                .map(|input_field| self.__input_value(input_field, shape_id)),
                        );
                        self.response.push_list(values).into()
                    }
                    _ => ResponseValue::Null,
                })
            }
        }
    }

    fn __type_fields(
        &self,
        field: &FieldShapeRecord,
        field_definitions: impl Iterator<Item = FieldDefinition<'ctx>>,
    ) -> ResponseValue {
        let shape_id = field.shape.as_concrete_object().unwrap();
        let include_deprecated = field
            .id
            .walk(&self.ctx)
            .hydrated_arguments(&self.ctx)
            .get_arg_value_as::<bool>("includeDeprecated");
        let mut values = self.response.new_list();
        values.extend(
            field_definitions
                .filter(|field| {
                    !field.is_inaccessible()
                        && (!is_deprecated(field.directives()) || include_deprecated)
                        && !self.metadata.meta_fields.contains(&field.id)
                })
                .map(|field| self.__field(field, shape_id)),
        );
        self.response.push_list(values).into()
    }

    fn __type_interfaces(
        &self,
        field: &FieldShapeRecord,
        interface_definitions: impl Iterator<Item = InterfaceDefinition<'ctx>>,
    ) -> ResponseValue {
        let shape_id = field.shape.as_concrete_object().unwrap();
        let mut values = self.response.new_list();
        values.extend(
            interface_definitions
                .filter(|inf| !inf.is_inaccessible())
                .map(|interface| self.__type_inner(Definition::Interface(interface), shape_id)),
        );
        self.response.push_list(values).into()
    }

    fn __type_possible_types(
        &self,
        field: &FieldShapeRecord,
        possible_types: impl Iterator<Item = ObjectDefinition<'ctx>>,
    ) -> ResponseValue {
        let shape_id = field.shape.as_concrete_object().unwrap();
        let mut values = self.response.new_list();
        values.extend(
            possible_types
                .filter(|obj| !obj.is_inaccessible())
                .map(|possible_type| self.__type_inner(Definition::Object(possible_type), shape_id)),
        );
        self.response.push_list(values).into()
    }

    fn __field(&self, target: FieldDefinition<'ctx>, shape_id: ConcreteShapeId) -> ResponseValue {
        self.object(&self.metadata.__field, shape_id, |field, __field| match __field {
            _Field::Name => target.as_ref().name_id.into(),
            _Field::Description => target.as_ref().description_id.into(),
            _Field::Args => {
                let shape_id = field.shape.as_concrete_object().unwrap();
                let mut values = self.response.new_list();
                values.extend(
                    target
                        .arguments()
                        .filter(|argument| !argument.is_inaccessible())
                        .map(|argument| self.__input_value(argument, shape_id)),
                );
                self.response.push_list(values).into()
            }
            _Field::Type => self.__type(target.ty(), field.shape.as_concrete_object().unwrap()),
            _Field::IsDeprecated => is_deprecated(target.directives()).into(),
            _Field::DeprecationReason => deprecation_reason(target.directives()).into(),
        })
    }

    fn __input_value(&self, target: InputValueDefinition<'ctx>, shape_id: ConcreteShapeId) -> ResponseValue {
        self.object(
            &self.metadata.__input_value,
            shape_id,
            |field, __input_value| match __input_value {
                __InputValue::Name => target.as_ref().name_id.into(),
                __InputValue::Description => target.as_ref().description_id.into(),
                __InputValue::Type => self.__type(target.ty(), field.shape.as_concrete_object().unwrap()),
                __InputValue::DefaultValue => target
                    .as_ref()
                    .default_value_id
                    .map(|id| self.schema.walk(&self.schema[id]).to_string())
                    .into(),
            },
        )
    }

    fn __enum_value(&self, target: EnumValue<'ctx>, shape_id: ConcreteShapeId) -> ResponseValue {
        self.object(
            &self.metadata.__enum_value,
            shape_id,
            |_, __enum_value| match __enum_value {
                __EnumValue::Name => target.as_ref().name_id.into(),
                __EnumValue::Description => target.as_ref().description_id.into(),
                __EnumValue::IsDeprecated => is_deprecated(target.directives()).into(),
                __EnumValue::DeprecationReason => deprecation_reason(target.directives()).into(),
            },
        )
    }
}

fn is_deprecated<'a>(mut directives: impl Iter<Item = TypeSystemDirective<'a>>) -> bool {
    directives.any(|d| matches!(d, TypeSystemDirective::Deprecated(_)))
}

fn deprecation_reason<'a>(mut directives: impl Iter<Item = TypeSystemDirective<'a>>) -> Option<StringId> {
    directives.find_map(|d| match d {
        TypeSystemDirective::Deprecated(reason) => reason.reason_id,
        _ => None,
    })
}
