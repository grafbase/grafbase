use std::cell::RefCell;

use schema::{
    EntityDefinition, EnumValue, FieldDefinition, InputValueDefinition, InterfaceDefinition, ListWrapping,
    MutableWrapping, ObjectDefinition, Schema, StringId, Type, TypeDefinition, TypeSystemDirective,
    introspection::{
        __EnumValue, __InputValue, __Schema, __Type, _Field, IntrospectionField, IntrospectionObject,
        IntrospectionSubgraph,
    },
};
use walker::{Iter, Walk};

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{ConcreteShapeId, FieldShapeRecord, Plan, RootFieldsShape, Shapes},
    response::{ResponseField, ResponseObject, ResponseObjectRef, ResponsePartBuilder, ResponseValue},
};

pub(super) struct IntrospectionWriter<'ctx, R: Runtime> {
    pub ctx: ExecutionContext<'ctx, R>,
    pub schema: &'ctx Schema,
    pub shapes: &'ctx Shapes,
    pub metadata: &'ctx IntrospectionSubgraph,
    pub plan: Plan<'ctx>,
    pub response: RefCell<ResponsePartBuilder<'ctx>>,
}

impl<'ctx, R: Runtime> IntrospectionWriter<'ctx, R> {
    pub(super) fn write(&self, parent_object: &ResponseObjectRef, shape: RootFieldsShape<'ctx>) {
        let shape = shape.concrete_shape();
        let mut fields = Vec::with_capacity(shape.field_shape_ids.len() + shape.typename_shape_ids.len());
        for field_shape in shape.field_shape_ids.walk(&self.ctx) {
            let field = field_shape.partition_field().as_data().unwrap();
            let arguments = field.arguments();
            match self.metadata.root_field(field.definition_id) {
                IntrospectionField::Type => {
                    let name = arguments.get_arg_value_as::<&str>("name", self.ctx.variables());
                    fields.push(ResponseField {
                        key: field_shape.key(),
                        value: self
                            .schema
                            .type_definition_by_name(name)
                            .filter(|def| !def.is_inaccessible())
                            .map(|definition| self.__type_inner(definition, field_shape.shape.as_concrete().unwrap()))
                            .into(),
                    });
                }
                IntrospectionField::Schema => {
                    fields.push(ResponseField {
                        key: field_shape.key(),
                        value: self.__schema(field_shape.shape.as_concrete().unwrap()),
                    });
                }
            };
        }
        if !shape.typename_shape_ids.is_empty() {
            let name_id = match self.plan.entity_definition() {
                EntityDefinition::Object(object) => object.name_id,
                EntityDefinition::Interface(interface) => interface.name_id,
            };
            for field in shape.typename_shape_ids.walk(&self.ctx) {
                fields.push(ResponseField {
                    key: field.key(),
                    value: name_id.into(),
                });
            }
        }
        self.response.borrow_mut().insert_fields_update(parent_object, fields);
    }

    fn object<E: Copy, const N: usize>(
        &self,
        object: &'ctx IntrospectionObject<E, N>,
        shape_id: ConcreteShapeId,
        build: impl Fn(&'ctx FieldShapeRecord, E) -> ResponseValue,
    ) -> ResponseValue {
        let shape = &self.shapes[shape_id];
        let mut fields = Vec::with_capacity(shape.field_shape_ids.len() + shape.typename_shape_ids.len());
        for field_shape in shape.field_shape_ids.walk(&self.ctx) {
            fields.push(ResponseField {
                key: field_shape.key(),
                value: build(
                    field_shape.as_ref(),
                    object[field_shape.partition_field().as_data().unwrap().definition_id],
                ),
            });
        }
        if !shape.typename_shape_ids.is_empty() {
            let name = self.schema.walk(object.id).as_ref().name_id;
            for field in shape.typename_shape_ids.walk(&self.ctx) {
                fields.push(ResponseField {
                    key: field.key(),
                    value: name.into(),
                });
            }
        }

        self.response
            .borrow_mut()
            .data
            .push_object(ResponseObject::new(Some(object.id), fields))
            .into()
    }

    fn __schema(&self, shape_id: ConcreteShapeId) -> ResponseValue {
        self.object(&self.metadata.__schema, shape_id, |field, __schema| {
            match __schema {
                __Schema::Description => self.schema.graph.description_id.into(),
                __Schema::Types => {
                    let shape_id = field.shape.as_concrete().unwrap();
                    let mut values = Vec::with_capacity(self.schema.type_definitions().len());
                    values.extend(
                        self.schema
                            .type_definitions()
                            .filter(|def| !def.is_inaccessible())
                            .map(|definition| self.__type_inner(definition, shape_id)),
                    );
                    let length = values.len() as u32;
                    let list_id = self.response.borrow_mut().data.push_list(values);
                    ResponseValue::List {
                        id: list_id,
                        offset: 0,
                        length,
                    }
                }
                __Schema::QueryType => self.__type_inner(
                    TypeDefinition::Object(self.schema.query()),
                    field.shape.as_concrete().unwrap(),
                ),
                __Schema::MutationType => self
                    .schema
                    .mutation()
                    .map(|mutation| {
                        self.__type_inner(TypeDefinition::Object(mutation), field.shape.as_concrete().unwrap())
                    })
                    .unwrap_or_default(),
                __Schema::SubscriptionType => self
                    .schema
                    .subscription()
                    .map(|subscription| {
                        self.__type_inner(TypeDefinition::Object(subscription), field.shape.as_concrete().unwrap())
                    })
                    .unwrap_or_default(),
                // TODO: Need to implemented directives...
                __Schema::Directives => {
                    let values = Vec::new();
                    let length = values.len() as u32;
                    let list_id = self.response.borrow_mut().data.push_list(values);
                    ResponseValue::List {
                        id: list_id,
                        offset: 0,
                        length,
                    }
                }
            }
        })
    }

    fn __type(&self, ty: Type<'ctx>, shape_id: ConcreteShapeId) -> ResponseValue {
        self.__type_list_wrapping(ty.definition(), ty.wrapping.into(), shape_id)
    }

    fn __type_list_wrapping(
        &self,
        definition: TypeDefinition<'ctx>,
        mut wrapping: MutableWrapping,
        shape_id: ConcreteShapeId,
    ) -> ResponseValue {
        match wrapping.pop_outermost_list_wrapping() {
            Some(list_wrapping) => match list_wrapping {
                ListWrapping::ListNonNull => {
                    wrapping.push_outermost_list_wrapping(ListWrapping::List);
                    self.__type_required_wrapping(definition, wrapping, shape_id)
                }
                ListWrapping::List => self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
                    __Type::Kind => self.metadata.type_kind.list.into(),
                    __Type::OfType => {
                        self.__type_list_wrapping(definition, wrapping.clone(), field.shape.as_concrete().unwrap())
                    }
                    _ => ResponseValue::Null,
                }),
            },
            None => {
                if wrapping.is_required() {
                    self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
                        __Type::Kind => self.metadata.type_kind.non_null.into(),
                        __Type::OfType => self.__type_inner(definition, field.shape.as_concrete().unwrap()),
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
        definition: TypeDefinition<'ctx>,
        wrapping: MutableWrapping,
        shape_id: ConcreteShapeId,
    ) -> ResponseValue {
        self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
            __Type::Kind => self.metadata.type_kind.non_null.into(),
            __Type::OfType => {
                self.__type_list_wrapping(definition, wrapping.clone(), field.shape.as_concrete().unwrap())
            }
            _ => ResponseValue::Null,
        })
    }

    fn __type_inner(&self, definition: TypeDefinition<'ctx>, shape_id: ConcreteShapeId) -> ResponseValue {
        match definition {
            TypeDefinition::Scalar(scalar) => self.object(&self.metadata.__type, shape_id, |_, __type| match __type {
                __Type::Kind => self.metadata.type_kind.scalar.into(),
                __Type::Name => scalar.name_id.into(),
                __Type::Description => scalar.description_id.into(),
                __Type::SpecifiedByURL => scalar.specified_by_url_id.into(),
                _ => ResponseValue::Null,
            }),
            TypeDefinition::Object(object) => {
                self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
                    __Type::Kind => self.metadata.type_kind.object.into(),
                    __Type::Name => object.name_id.into(),
                    __Type::Description => object.description_id.into(),
                    __Type::Fields => self.__type_fields(field, object.fields()),
                    __Type::Interfaces => self.__type_interfaces(field, object.interfaces()),
                    _ => ResponseValue::Null,
                })
            }
            TypeDefinition::Interface(interface) => {
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
            TypeDefinition::Union(union) => {
                self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
                    __Type::Kind => self.metadata.type_kind.union.into(),
                    __Type::Name => union.name_id.into(),
                    __Type::Description => union.description_id.into(),
                    __Type::PossibleTypes => self.__type_possible_types(field, union.possible_types()),
                    _ => ResponseValue::Null,
                })
            }
            TypeDefinition::Enum(r#enum) => {
                self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
                    __Type::Kind => self.metadata.type_kind.r#enum.into(),
                    __Type::Name => r#enum.name_id.into(),
                    __Type::Description => r#enum.description_id.into(),
                    __Type::EnumValues => {
                        let shape_id = field.shape.as_concrete().unwrap();
                        let include_deprecated = field
                            .id
                            .as_data()
                            .unwrap()
                            .walk(&self.ctx)
                            .arguments()
                            .get_arg_value_as::<bool>("includeDeprecated", self.ctx.variables());
                        let mut values = Vec::with_capacity(r#enum.value_ids.len());
                        values.extend(
                            r#enum
                                .values()
                                .filter(|value| {
                                    !value.is_inaccessible()
                                        && (!is_deprecated(value.directives()) || include_deprecated)
                                })
                                .map(|value| self.__enum_value(value, shape_id)),
                        );
                        let length = values.len() as u32;
                        let list_id = self.response.borrow_mut().data.push_list(values);
                        ResponseValue::List {
                            id: list_id,
                            offset: 0,
                            length,
                        }
                    }
                    _ => ResponseValue::Null,
                })
            }
            TypeDefinition::InputObject(input_object) => {
                self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
                    __Type::Kind => self.metadata.type_kind.input_object.into(),
                    __Type::Name => input_object.name_id.into(),
                    __Type::Description => input_object.description_id.into(),
                    __Type::InputFields => {
                        let shape_id = field.shape.as_concrete().unwrap();
                        let mut values = Vec::with_capacity(input_object.input_field_ids.len());
                        values.extend(
                            input_object
                                .input_fields()
                                .filter(|input_field| !input_field.is_inaccessible())
                                .map(|input_field| self.__input_value(input_field, shape_id)),
                        );
                        let length = values.len() as u32;
                        let list_id = self.response.borrow_mut().data.push_list(values);
                        ResponseValue::List {
                            id: list_id,
                            offset: 0,
                            length,
                        }
                    }
                    _ => ResponseValue::Null,
                })
            }
        }
    }

    fn __type_fields(
        &self,
        field: &FieldShapeRecord,
        field_definitions: impl Iter<Item = FieldDefinition<'ctx>>,
    ) -> ResponseValue {
        let shape_id = field.shape.as_concrete().unwrap();
        let include_deprecated = field
            .id
            .as_data()
            .unwrap()
            .walk(&self.ctx)
            .arguments()
            .get_arg_value_as::<bool>("includeDeprecated", self.ctx.variables());
        let mut values = Vec::with_capacity(field_definitions.len());
        values.extend(
            field_definitions
                .filter(|field| {
                    !field.is_inaccessible()
                        && (!is_deprecated(field.directives()) || include_deprecated)
                        && !self.metadata.meta_fields.contains(&field.id)
                })
                .map(|field| self.__field(field, shape_id)),
        );
        let length = values.len() as u32;
        let list_id = self.response.borrow_mut().data.push_list(values);
        ResponseValue::List {
            id: list_id,
            offset: 0,
            length,
        }
    }

    fn __type_interfaces(
        &self,
        field: &FieldShapeRecord,
        interface_definitions: impl Iter<Item = InterfaceDefinition<'ctx>>,
    ) -> ResponseValue {
        let shape_id = field.shape.as_concrete().unwrap();
        let mut values = Vec::with_capacity(interface_definitions.len());
        values.extend(
            interface_definitions
                .filter(|inf| !inf.is_inaccessible())
                .map(|interface| self.__type_inner(TypeDefinition::Interface(interface), shape_id)),
        );
        let length = values.len() as u32;
        let list_id = self.response.borrow_mut().data.push_list(values);
        ResponseValue::List {
            id: list_id,
            offset: 0,
            length,
        }
    }

    fn __type_possible_types(
        &self,
        field: &FieldShapeRecord,
        possible_types: impl Iter<Item = ObjectDefinition<'ctx>>,
    ) -> ResponseValue {
        let shape_id = field.shape.as_concrete().unwrap();
        let mut values = Vec::with_capacity(possible_types.len());
        values.extend(
            possible_types
                .filter(|obj| !obj.is_inaccessible())
                .map(|possible_type| self.__type_inner(TypeDefinition::Object(possible_type), shape_id)),
        );
        let length = values.len() as u32;
        let list_id = self.response.borrow_mut().data.push_list(values);
        ResponseValue::List {
            id: list_id,
            offset: 0,
            length,
        }
    }

    fn __field(&self, target: FieldDefinition<'ctx>, shape_id: ConcreteShapeId) -> ResponseValue {
        self.object(&self.metadata.__field, shape_id, |field, __field| match __field {
            _Field::Name => target.as_ref().name_id.into(),
            _Field::Description => target.as_ref().description_id.into(),
            _Field::Args => {
                let shape_id = field.shape.as_concrete().unwrap();
                let mut values = Vec::with_capacity(target.argument_ids.len());
                values.extend(
                    target
                        .arguments()
                        .filter(|argument| !argument.is_inaccessible())
                        .map(|argument| self.__input_value(argument, shape_id)),
                );
                let length = values.len() as u32;
                let list_id = self.response.borrow_mut().data.push_list(values);
                ResponseValue::List {
                    id: list_id,
                    offset: 0,
                    length,
                }
            }
            _Field::Type => self.__type(target.ty(), field.shape.as_concrete().unwrap()),
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
                __InputValue::Type => self.__type(target.ty(), field.shape.as_concrete().unwrap()),
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
