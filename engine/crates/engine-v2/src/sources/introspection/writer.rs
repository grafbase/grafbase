use readable::Iter;
use schema::{
    introspection::{
        IntrospectionField, IntrospectionMetadata, IntrospectionObject, _Field, __EnumValue, __InputValue, __Schema,
        __Type,
    },
    Definition, EntityDefinition, EnumValue, FieldDefinition, InputValueDefinition, InterfaceDefinition, ListWrapping,
    ObjectDefinition, Schema, StringId, Type, TypeSystemDirective, Wrapping,
};

use crate::{
    operation::{PlanField, PlanWalker},
    response::{
        ConcreteObjectShapeId, FieldShape, ResponseObject, ResponseObjectField, ResponseValue, ResponseWriter, Shapes,
    },
};

pub(super) struct IntrospectionWriter<'a> {
    pub schema: &'a Schema,
    pub metadata: &'a IntrospectionMetadata,
    pub shapes: &'a Shapes,
    pub plan: PlanWalker<'a, ()>,
    pub response: ResponseWriter<'a>,
}

impl<'a> IntrospectionWriter<'a> {
    pub(super) fn execute(self, id: ConcreteObjectShapeId) {
        let shape = &self.shapes[id];
        let mut fields = Vec::with_capacity(shape.field_shape_ids.len() + shape.typename_response_edges.len());
        for id in shape.field_shape_ids {
            let FieldShape {
                id,
                definition_id,
                shape,
                edge,
                ..
            } = &self.shapes[id];
            let field = self.plan.walk(*id);
            match self.metadata.root_field(*definition_id) {
                IntrospectionField::Type => {
                    let name = field.get_arg_value_as::<&str>("name");
                    fields.push(ResponseObjectField {
                        edge: *edge,
                        required_field_id: None,
                        value: self
                            .schema
                            .definition_by_name(name)
                            .map(|definition| {
                                self.__type_inner(self.schema.walk(definition), shape.as_concrete_object().unwrap())
                            })
                            .into(),
                    });
                }
                IntrospectionField::Schema => {
                    fields.push(ResponseObjectField {
                        edge: *edge,
                        required_field_id: None,
                        value: self.__schema(shape.as_concrete_object().unwrap()),
                    });
                }
            };
        }
        if !shape.typename_response_edges.is_empty() {
            let name_id = match self.schema.walk(self.plan.logical_plan().as_ref().entity_id) {
                EntityDefinition::Object(object) => object.name_id,
                EntityDefinition::Interface(interface) => interface.name_id,
            };
            for edge in &shape.typename_response_edges {
                fields.push(ResponseObjectField {
                    edge: *edge,
                    required_field_id: None,
                    value: name_id.into(),
                });
            }
        }
        self.response.update_root_object_with(fields);
    }

    fn walk(&self, field: &FieldShape) -> PlanField<'a> {
        self.plan.walk(field.id)
    }

    fn object<E: Copy, const N: usize>(
        &self,
        object: &'a IntrospectionObject<E, N>,
        shape_id: ConcreteObjectShapeId,
        build: impl Fn(&'a FieldShape, E) -> ResponseValue,
    ) -> ResponseValue {
        let shape = &self.shapes[shape_id];
        let mut fields = Vec::with_capacity(shape.field_shape_ids.len() + shape.typename_response_edges.len());
        for id in shape.field_shape_ids {
            let field = &self.shapes[id];
            fields.push(ResponseObjectField {
                edge: field.edge,
                required_field_id: None,
                value: build(field, object[field.definition_id]),
            });
        }
        if !shape.typename_response_edges.is_empty() {
            let name = self.schema.walk(object.id).as_ref().name_id;
            for edge in &shape.typename_response_edges {
                fields.push(ResponseObjectField {
                    edge: *edge,
                    required_field_id: None,
                    value: name.into(),
                });
            }
        }

        self.response.push_object(ResponseObject::new(fields)).into()
    }

    fn __schema(&self, shape_id: ConcreteObjectShapeId) -> ResponseValue {
        self.object(&self.metadata.__schema, shape_id, |field, __schema| {
            match __schema {
                __Schema::Description => self.schema.graph.description_id.into(),
                __Schema::Types => {
                    let shape_id = field.shape.as_concrete_object().unwrap();
                    let mut values = self.response.new_list();
                    values.extend(
                        self.schema
                            .definitions()
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

    fn __type(&self, ty: Type<'a>, shape_id: ConcreteObjectShapeId) -> ResponseValue {
        self.__type_list_wrapping(ty.definition(), ty.wrapping, shape_id)
    }

    fn __type_list_wrapping(
        &self,
        definition: Definition<'a>,
        mut wrapping: Wrapping,
        shape_id: ConcreteObjectShapeId,
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
        definition: Definition<'a>,
        wrapping: Wrapping,
        shape_id: ConcreteObjectShapeId,
    ) -> ResponseValue {
        self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
            __Type::Kind => self.metadata.type_kind.non_null.into(),
            __Type::OfType => {
                self.__type_list_wrapping(definition, wrapping, field.shape.as_concrete_object().unwrap())
            }
            _ => ResponseValue::Null,
        })
    }

    fn __type_inner(&self, definition: Definition<'a>, shape_id: ConcreteObjectShapeId) -> ResponseValue {
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
                    let include_deprecated = self.walk(field).get_arg_value_as::<bool>("includeDeprecated");
                    let mut values = self.response.new_list();
                    values.extend(
                        r#enum
                            .values()
                            .filter(|value| (!is_deprecated(value.directives()) || include_deprecated))
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
        field: &FieldShape,
        field_definitions: impl Iter<Item = FieldDefinition<'a>>,
    ) -> ResponseValue {
        let shape_id = field.shape.as_concrete_object().unwrap();
        let include_deprecated = self.walk(field).get_arg_value_as::<bool>("includeDeprecated");
        let mut values = self.response.new_list();
        values.extend(
            field_definitions
                .filter(|field| {
                    (!is_deprecated(field.directives()) || include_deprecated)
                        && !self.metadata.meta_fields.contains(&field.id())
                })
                .map(|field| self.__field(field, shape_id)),
        );
        self.response.push_list(values).into()
    }

    fn __type_interfaces(
        &self,
        field: &FieldShape,
        interface_definitions: impl Iter<Item = InterfaceDefinition<'a>>,
    ) -> ResponseValue {
        let shape_id = field.shape.as_concrete_object().unwrap();
        let mut values = self.response.new_list();
        values.extend(
            interface_definitions.map(|interface| self.__type_inner(Definition::Interface(interface), shape_id)),
        );
        self.response.push_list(values).into()
    }

    fn __type_possible_types(
        &self,
        field: &FieldShape,
        possible_types: impl Iter<Item = ObjectDefinition<'a>>,
    ) -> ResponseValue {
        let shape_id = field.shape.as_concrete_object().unwrap();
        let mut values = self.response.new_list();
        values
            .extend(possible_types.map(|possible_type| self.__type_inner(Definition::Object(possible_type), shape_id)));
        self.response.push_list(values).into()
    }

    fn __field(&self, target: FieldDefinition<'a>, shape_id: ConcreteObjectShapeId) -> ResponseValue {
        self.object(&self.metadata.__field, shape_id, |field, __field| match __field {
            _Field::Name => target.as_ref().name_id.into(),
            _Field::Description => target.as_ref().description_id.into(),
            _Field::Args => {
                let shape_id = field.shape.as_concrete_object().unwrap();
                let mut values = self.response.new_list();
                values.extend(
                    target
                        .arguments()
                        .map(|argument| self.__input_value(argument, shape_id)),
                );
                self.response.push_list(values).into()
            }
            _Field::Type => self.__type(target.ty(), field.shape.as_concrete_object().unwrap()),
            _Field::IsDeprecated => is_deprecated(target.directives()).into(),
            _Field::DeprecationReason => deprecation_reason(target.directives()).into(),
        })
    }

    fn __input_value(&self, target: InputValueDefinition<'a>, shape_id: ConcreteObjectShapeId) -> ResponseValue {
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

    fn __enum_value(&self, target: EnumValue<'a>, shape_id: ConcreteObjectShapeId) -> ResponseValue {
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
