use schema::{
    sources::{
        introspection::{IntrospectionField, IntrospectionObject, _Field, __EnumValue, __InputValue, __Schema, __Type},
        IntrospectionMetadata,
    },
    Definition, DefinitionWalker, EnumValueWalker, FieldDefinitionWalker, InputValueDefinitionWalker, ListWrapping,
    SchemaWalker, TypeWalker, Wrapping,
};

use crate::{
    execution::{PlanField, PlanWalker},
    response::{ConcreteObjectShapeId, FieldShape, ResponseObject, ResponseValue, ResponseWriter, Shapes},
};

pub(super) struct IntrospectionWriter<'a> {
    pub schema: SchemaWalker<'a, ()>,
    pub metadata: &'a IntrospectionMetadata,
    pub shapes: &'a Shapes,
    pub plan: PlanWalker<'a, (), ()>,
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
            let field = self.plan.walk_with(*id, *definition_id);
            match self.metadata.root_field(*definition_id) {
                IntrospectionField::Type => {
                    let name = field.get_arg_value_as::<&str>("name");
                    fields.push((
                        *edge,
                        self.schema
                            .definition_by_name(name)
                            .map(|definition| {
                                self.__type_inner(self.schema.walk(definition), shape.as_concrete_object().unwrap())
                            })
                            .into(),
                    ));
                }
                IntrospectionField::Schema => {
                    fields.push((*edge, self.__schema(shape.as_concrete_object().unwrap())));
                }
            };
        }
        if !shape.typename_response_edges.is_empty() {
            let name = self
                .schema
                .walk(self.plan.logical_plan().as_ref().entity_id)
                .schema_name_id();
            for edge in &shape.typename_response_edges {
                fields.push((*edge, name.into()));
            }
        }
        self.response.update_root_object_with(fields);
    }

    fn walk(&self, field: &FieldShape) -> PlanField<'a> {
        self.plan.walk_with(field.id, field.definition_id)
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
            fields.push((field.edge, build(field, object[field.definition_id])));
        }
        if !shape.typename_response_edges.is_empty() {
            let name = self.schema.walk(object.id).as_ref().name;
            for edge in &shape.typename_response_edges {
                fields.push((*edge, name.into()));
            }
        }

        self.response.push_object(ResponseObject::new(fields)).into()
    }

    fn __schema(&self, shape_id: ConcreteObjectShapeId) -> ResponseValue {
        self.object(&self.metadata.__schema, shape_id, |field, __schema| {
            match __schema {
                __Schema::Description => self.schema.description_id().into(),
                __Schema::Types => {
                    let shape_id = field.shape.as_concrete_object().unwrap();
                    let values = self
                        .schema
                        .definitions()
                        .map(|definition| self.__type_inner(definition, shape_id))
                        .collect::<Vec<_>>();
                    self.response.push_list(&values).into()
                }
                __Schema::QueryType => {
                    self.__type_inner(self.schema.query().into(), field.shape.as_concrete_object().unwrap())
                }
                __Schema::MutationType => self
                    .schema
                    .mutation()
                    .map(|mutation| self.__type_inner(mutation.into(), field.shape.as_concrete_object().unwrap()))
                    .unwrap_or_default(),
                __Schema::SubscriptionType => self
                    .schema
                    .subscription()
                    .map(|subscription| {
                        self.__type_inner(subscription.into(), field.shape.as_concrete_object().unwrap())
                    })
                    .unwrap_or_default(),
                // TODO: Need to implemented directives...
                __Schema::Directives => self.response.push_list(&[]).into(),
            }
        })
    }

    fn __type(&self, ty: TypeWalker<'a>, shape_id: ConcreteObjectShapeId) -> ResponseValue {
        self.__type_list_wrapping(ty.inner(), ty.wrapping(), shape_id)
    }

    fn __type_list_wrapping(
        &self,
        definition: DefinitionWalker<'a>,
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
        definition: DefinitionWalker<'a>,
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

    fn __type_inner(&self, definition: DefinitionWalker<'a>, shape_id: ConcreteObjectShapeId) -> ResponseValue {
        self.object(&self.metadata.__type, shape_id, |field, __type| match __type {
            __Type::Kind => match definition.id() {
                Definition::Scalar(_) => self.metadata.type_kind.scalar,
                Definition::Object(_) => self.metadata.type_kind.object,
                Definition::Interface(_) => self.metadata.type_kind.interface,
                Definition::Union(_) => self.metadata.type_kind.union,
                Definition::Enum(_) => self.metadata.type_kind.r#enum,
                Definition::InputObject(_) => self.metadata.type_kind.input_object,
            }
            .into(),
            __Type::Name => definition.schema_name_id().into(),
            __Type::Description => definition.schema_description_id().into(),
            __Type::Fields => definition
                .fields()
                .map(|fields| {
                    let shape_id = field.shape.as_concrete_object().unwrap();
                    let include_deprecated = self.walk(field).get_arg_value_as::<bool>("includeDeprecated");
                    let values = fields
                        .filter(|field| {
                            (!field.directives().has_deprecated() || include_deprecated)
                                && !self.metadata.meta_fields.contains(&field.id())
                        })
                        .map(|field| self.__field(field, shape_id))
                        .collect::<Vec<_>>();
                    self.response.push_list(&values)
                })
                .into(),
            __Type::Interfaces => definition
                .interfaces()
                .map(|interfaces| {
                    let shape_id = field.shape.as_concrete_object().unwrap();
                    let values = interfaces
                        .map(|interface| self.__type_inner(interface.into(), shape_id))
                        .collect::<Vec<_>>();
                    self.response.push_list(&values)
                })
                .into(),
            __Type::PossibleTypes => definition
                .possible_types()
                .map(|possible_types| {
                    let shape_id = field.shape.as_concrete_object().unwrap();
                    let values = possible_types
                        .map(|interface| self.__type_inner(interface.into(), shape_id))
                        .collect::<Vec<_>>();
                    self.response.push_list(&values)
                })
                .into(),
            __Type::EnumValues => definition
                .as_enum()
                .map(|r#enum| {
                    let shape_id = field.shape.as_concrete_object().unwrap();
                    let include_deprecated = self.walk(field).get_arg_value_as::<bool>("includeDeprecated");
                    let values = r#enum
                        .values()
                        .filter(|value| (!value.directives().has_deprecated() || include_deprecated))
                        .map(|value| self.__enum_value(value, shape_id))
                        .collect::<Vec<_>>();
                    self.response.push_list(&values)
                })
                .into(),
            __Type::InputFields => definition
                .as_input_object()
                .map(|input_object| {
                    let shape_id = field.shape.as_concrete_object().unwrap();
                    let values = input_object
                        .input_fields()
                        .map(|input_field| self.__input_value(input_field, shape_id))
                        .collect::<Vec<_>>();
                    self.response.push_list(&values)
                })
                .into(),
            __Type::OfType => ResponseValue::Null,
            __Type::SpecifiedByURL => definition
                .as_scalar()
                .and_then(|scalar| scalar.as_ref().specified_by_url)
                .into(),
        })
    }

    fn __field(&self, target: FieldDefinitionWalker<'a>, shape_id: ConcreteObjectShapeId) -> ResponseValue {
        self.object(&self.metadata.__field, shape_id, |field, __field| match __field {
            _Field::Name => target.as_ref().name.into(),
            _Field::Description => target.as_ref().description.into(),
            _Field::Args => {
                let shape_id = field.shape.as_concrete_object().unwrap();
                let values = target
                    .arguments()
                    .map(|argument| self.__input_value(argument, shape_id))
                    .collect::<Vec<_>>();

                self.response.push_list(&values).into()
            }
            _Field::Type => self.__type(target.ty(), field.shape.as_concrete_object().unwrap()),
            _Field::IsDeprecated => target.directives().has_deprecated().into(),
            _Field::DeprecationReason => target.directives().deprecated().map(|d| d.reason).into(),
        })
    }

    fn __input_value(&self, target: InputValueDefinitionWalker<'a>, shape_id: ConcreteObjectShapeId) -> ResponseValue {
        self.object(
            &self.metadata.__input_value,
            shape_id,
            |field, __input_value| match __input_value {
                __InputValue::Name => target.as_ref().name.into(),
                __InputValue::Description => target.as_ref().description.into(),
                __InputValue::Type => self.__type(target.ty(), field.shape.as_concrete_object().unwrap()),
                __InputValue::DefaultValue => target
                    .as_ref()
                    .default_value
                    .map(|id| self.schema.walk(&self.schema[id]).to_string())
                    .into(),
            },
        )
    }

    fn __enum_value(&self, target: EnumValueWalker<'a>, shape_id: ConcreteObjectShapeId) -> ResponseValue {
        self.object(
            &self.metadata.__enum_value,
            shape_id,
            |_, __enum_value| match __enum_value {
                __EnumValue::Name => target.as_ref().name.into(),
                __EnumValue::Description => target.as_ref().description.into(),
                __EnumValue::IsDeprecated => target.directives().has_deprecated().into(),
                __EnumValue::DeprecationReason => target.directives().deprecated().map(|d| d.reason).into(),
            },
        )
    }
}
