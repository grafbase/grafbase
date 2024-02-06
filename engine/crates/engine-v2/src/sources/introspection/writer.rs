use std::{cell::RefCell, collections::BTreeMap};

use engine_value::ConstValue;
use schema::{
    sources::introspection::{
        IntrospectionField, IntrospectionObject, Metadata, __EnumValue, __Field, __InputValue, __Schema, __Type,
    },
    Definition, DefinitionWalker, EnumValue, FieldWalker, InputValueWalker, ListWrapping, SchemaWalker, TypeWalker,
    Wrapping,
};

use crate::{
    plan::{CollectedField, PlanCollectedField, PlanCollectedSelectionSet, PlanWalker},
    response::{ResponseBoundaryItem, ResponseObject, ResponseObjectUpdate, ResponsePart, ResponseValue},
};

pub(super) struct IntrospectionWriter<'a> {
    pub schema: SchemaWalker<'a, ()>,
    pub metadata: &'a Metadata,
    pub plan: PlanWalker<'a>,
    pub output: RefCell<&'a mut ResponsePart>,
}

impl<'a> IntrospectionWriter<'a> {
    pub(super) fn update_output(&self, response_object: ResponseBoundaryItem) {
        let mut fields = BTreeMap::new();
        let selection_set = self.plan.collected_selection_set();
        for field in selection_set.fields() {
            let &CollectedField {
                edge, schema_field_id, ..
            } = field.as_ref();
            match self.metadata.root_field(schema_field_id) {
                IntrospectionField::Type => {
                    let name = field
                        .as_bound_field()
                        .arguments()
                        .next()
                        .map(|arg| match arg.resolved_value() {
                            ConstValue::String(s) => s,
                            _ => unreachable!("Validation failure: Expected string argument"),
                        })
                        .expect("Validation failure: missing argument");
                    fields.insert(
                        edge,
                        self.schema
                            .definition_by_name(&name)
                            .map(|definition| {
                                self.__type_inner(self.schema.walk(definition), field.concrete_selection_set().unwrap())
                            })
                            .into(),
                    );
                }
                IntrospectionField::Schema => {
                    fields.insert(edge, self.__schema(field.concrete_selection_set().unwrap()));
                }
            };
        }
        if !selection_set.as_ref().typename_fields.is_empty() {
            let name = selection_set.ty().schema_name_id();
            for edge in &selection_set.as_ref().typename_fields {
                fields.insert(*edge, name.into());
            }
        }
        self.output.borrow_mut().push_update(ResponseObjectUpdate {
            id: response_object.response_object_id,
            fields,
        });
    }

    fn object<E: Copy, const N: usize>(
        &self,
        object: &'a IntrospectionObject<E, N>,
        selection_set: PlanCollectedSelectionSet<'_>,
        build: impl Fn(PlanCollectedField<'_>, E) -> ResponseValue,
    ) -> ResponseValue {
        let mut fields = BTreeMap::new();
        for field in selection_set.fields() {
            let &CollectedField {
                edge, schema_field_id, ..
            } = field.as_ref();
            fields.insert(edge, build(field, object[schema_field_id]));
        }
        if !selection_set.as_ref().typename_fields.is_empty() {
            let name = selection_set.ty().schema_name_id();
            for edge in &selection_set.as_ref().typename_fields {
                fields.insert(*edge, name.into());
            }
        }

        self.output.borrow_mut().push_object(ResponseObject { fields }).into()
    }

    fn __schema(&self, selection_set: PlanCollectedSelectionSet<'_>) -> ResponseValue {
        self.object(&self.metadata.__schema, selection_set, |field, __schema| {
            match __schema {
                __Schema::Description => self.schema.description.into(),
                __Schema::Types => {
                    let selection_set = field.concrete_selection_set().unwrap();
                    let values = self
                        .schema
                        .definitions()
                        .map(|definition| self.__type_inner(definition, selection_set))
                        .collect::<Vec<_>>();
                    self.output.borrow_mut().push_list(&values).into()
                }
                __Schema::QueryType => {
                    self.__type_inner(self.schema.query().into(), field.concrete_selection_set().unwrap())
                }
                __Schema::MutationType => self
                    .schema
                    .mutation()
                    .map(|mutation| self.__type_inner(mutation.into(), field.concrete_selection_set().unwrap()))
                    .unwrap_or_default(),
                __Schema::SubscriptionType => self
                    .schema
                    .subscription()
                    .map(|subscription| self.__type_inner(subscription.into(), field.concrete_selection_set().unwrap()))
                    .unwrap_or_default(),
                // TODO: Need to implemented directives...
                __Schema::Directives => self.output.borrow_mut().push_list(&[]).into(),
            }
        })
    }

    fn __type(&self, ty: TypeWalker<'a>, selection_set: PlanCollectedSelectionSet<'_>) -> ResponseValue {
        self.__type_list_wrapping(ty.inner(), ty.wrapping(), selection_set)
    }

    fn __type_list_wrapping(
        &self,
        definition: DefinitionWalker<'a>,
        mut wrapping: Wrapping,
        selection_set: PlanCollectedSelectionSet<'_>,
    ) -> ResponseValue {
        match wrapping.pop_list_wrapping() {
            Some(list_wrapping) => match list_wrapping {
                ListWrapping::RequiredList => {
                    self.__type_required_wrapping(definition, wrapping.wrapped_by_nullable_list(), selection_set)
                }
                ListWrapping::NullableList => {
                    self.object(&self.metadata.__type, selection_set, |field, __type| match __type {
                        __Type::Kind => self.metadata.type_kind.list.into(),
                        __Type::OfType => {
                            self.__type_list_wrapping(definition, wrapping, field.concrete_selection_set().unwrap())
                        }
                        _ => ResponseValue::Null,
                    })
                }
            },
            None => {
                if wrapping.inner_is_required() {
                    self.object(&self.metadata.__type, selection_set, |field, __type| match __type {
                        __Type::Kind => self.metadata.type_kind.non_null.into(),
                        __Type::OfType => self.__type_inner(definition, field.concrete_selection_set().unwrap()),
                        _ => ResponseValue::Null,
                    })
                } else {
                    self.__type_inner(definition, selection_set)
                }
            }
        }
    }

    fn __type_required_wrapping(
        &self,
        definition: DefinitionWalker<'a>,
        wrapping: Wrapping,
        selection_set: PlanCollectedSelectionSet<'_>,
    ) -> ResponseValue {
        self.object(&self.metadata.__type, selection_set, |field, __type| match __type {
            __Type::Kind => self.metadata.type_kind.non_null.into(),
            __Type::OfType => self.__type_list_wrapping(definition, wrapping, field.concrete_selection_set().unwrap()),
            _ => ResponseValue::Null,
        })
    }

    fn __type_inner(
        &self,
        definition: DefinitionWalker<'a>,
        selection_set: PlanCollectedSelectionSet<'_>,
    ) -> ResponseValue {
        self.object(&self.metadata.__type, selection_set, |field, __type| match __type {
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
                    let selection_set = field.concrete_selection_set().unwrap();
                    let include_deprecated = field
                        .as_bound_field()
                        .arguments()
                        .next()
                        .map(|arg| match arg.resolved_value() {
                            ConstValue::Boolean(b) => b,
                            _ => unreachable!("Validation failure: Expected boolean argument"),
                        })
                        .unwrap_or_default();
                    let values = fields
                        .filter(move |field| {
                            (!field.as_ref().is_deprecated || include_deprecated)
                                && !self.metadata.meta_fields.contains(&field.id())
                        })
                        .map(|field| self.__field(field, selection_set))
                        .collect::<Vec<_>>();
                    self.output.borrow_mut().push_list(&values)
                })
                .into(),
            __Type::Interfaces => definition
                .interfaces()
                .map(|interfaces| {
                    let selection_set = field.concrete_selection_set().unwrap();
                    let values = interfaces
                        .map(|interface| self.__type_inner(interface.into(), selection_set))
                        .collect::<Vec<_>>();
                    self.output.borrow_mut().push_list(&values)
                })
                .into(),
            __Type::PossibleTypes => definition
                .possible_types()
                .map(|possible_types| {
                    let selection_set = field.concrete_selection_set().unwrap();
                    let values = possible_types
                        .map(|interface| self.__type_inner(interface.into(), selection_set))
                        .collect::<Vec<_>>();
                    self.output.borrow_mut().push_list(&values)
                })
                .into(),
            __Type::EnumValues => definition
                .as_enum()
                .map(|r#enum| {
                    let selection_set = field.concrete_selection_set().unwrap();
                    let values = r#enum
                        .values()
                        .map(|value| self.__enum_value(value, selection_set))
                        .collect::<Vec<_>>();
                    self.output.borrow_mut().push_list(&values)
                })
                .into(),
            __Type::InputFields => definition
                .as_input_object()
                .map(|input_object| {
                    let selection_set = field.concrete_selection_set().unwrap();
                    let values = input_object
                        .input_fields()
                        .map(|input_field| self.__input_value(input_field, selection_set))
                        .collect::<Vec<_>>();
                    self.output.borrow_mut().push_list(&values)
                })
                .into(),
            __Type::OfType => ResponseValue::Null,
            __Type::SpecifiedByURL => definition
                .as_scalar()
                .and_then(|scalar| scalar.as_ref().specified_by_url)
                .into(),
        })
    }

    fn __field(&self, target: FieldWalker<'a>, selection_set: PlanCollectedSelectionSet<'_>) -> ResponseValue {
        self.object(&self.metadata.__field, selection_set, |field, __field| match __field {
            __Field::Name => target.as_ref().name.into(),
            __Field::Description => target.as_ref().description.into(),
            __Field::Args => {
                let selection_set = field.concrete_selection_set().unwrap();
                let values = target
                    .arguments()
                    .map(|argument| self.__input_value(argument, selection_set))
                    .collect::<Vec<_>>();

                self.output.borrow_mut().push_list(&values).into()
            }
            __Field::Type => self.__type(target.ty(), field.concrete_selection_set().unwrap()),
            __Field::IsDeprecated => target.as_ref().is_deprecated.into(),
            __Field::DeprecationReason => target.as_ref().deprecation_reason.into(),
        })
    }

    fn __input_value(
        &self,
        target: InputValueWalker<'a>,
        selection_set: PlanCollectedSelectionSet<'_>,
    ) -> ResponseValue {
        self.object(
            &self.metadata.__input_value,
            selection_set,
            |field, __input_value| match __input_value {
                __InputValue::Name => target.as_ref().name.into(),
                __InputValue::Description => target.as_ref().description.into(),
                __InputValue::Type => self.__type(target.ty(), field.concrete_selection_set().unwrap()),
                // TODO: default value...
                __InputValue::DefaultValue => ResponseValue::Null,
            },
        )
    }

    fn __enum_value(&self, target: &'a EnumValue, selection_set: PlanCollectedSelectionSet<'_>) -> ResponseValue {
        self.object(
            &self.metadata.__enum_value,
            selection_set,
            |_, __enum_value| match __enum_value {
                __EnumValue::Name => target.name.into(),
                __EnumValue::Description => target.description.into(),
                __EnumValue::IsDeprecated => target.is_deprecated.into(),
                __EnumValue::DeprecationReason => target.deprecation_reason.into(),
            },
        )
    }
}
