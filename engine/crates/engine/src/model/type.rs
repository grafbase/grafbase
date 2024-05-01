use std::collections::HashSet;

use meta_type_name::MetaTypeName;

use crate::{
    model::{__EnumValue, __Field, __InputValue, __TypeKind},
    registry, ContextField, Object,
};

enum TypeDetail<'a> {
    Named(registry_v2::MetaType<'a>),
    NonNull(String),
    List(String),
}

pub struct __Type<'a> {
    registry: &'a registry_v2::Registry,
    detail: TypeDetail<'a>,
}

impl<'a> __Type<'a> {
    #[inline]
    pub fn new_simple(registry: &'a registry_v2::Registry, ty: registry_v2::MetaType<'a>) -> __Type<'a> {
        __Type {
            registry,
            detail: TypeDetail::Named(ty),
        }
    }

    #[inline]
    pub fn new(registry: &'a registry_v2::Registry, type_name: &str) -> __Type<'a> {
        match MetaTypeName::create(type_name) {
            MetaTypeName::NonNull(ty) => __Type {
                registry,
                detail: TypeDetail::NonNull(ty.to_string()),
            },
            MetaTypeName::List(ty) => __Type {
                registry,
                detail: TypeDetail::List(ty.to_string()),
            },
            MetaTypeName::Named(ty) => __Type {
                registry,
                detail: TypeDetail::Named(match registry.lookup_type(ty) {
                    Some(t) => t,
                    None => panic!("Type '{ty}' not found!"),
                }),
            },
        }
    }
}

/// The fundamental unit of any GraphQL Schema is the type. There are many kinds of types in GraphQL as represented by the `__TypeKind` enum.
///
/// Depending on the kind of a type, certain fields describe information about that type. Scalar types provide no information beyond a name and description, while Enum types provide their values. Object and Interface types provide the fields they describe. Abstract types, Union and Interface, provide the Object types possible at runtime. List and NonNull types compose other types.
#[Object(internal, name = "__Type")]
impl<'a> __Type<'a> {
    #[inline]
    async fn kind(&self) -> __TypeKind {
        match &self.detail {
            TypeDetail::Named(ty) => match ty {
                registry_v2::MetaType::Scalar { .. } => __TypeKind::Scalar,
                registry_v2::MetaType::Object { .. } => __TypeKind::Object,
                registry_v2::MetaType::Interface { .. } => __TypeKind::Interface,
                registry_v2::MetaType::Union { .. } => __TypeKind::Union,
                registry_v2::MetaType::Enum { .. } => __TypeKind::Enum,
                registry_v2::MetaType::InputObject { .. } => __TypeKind::InputObject,
            },
            TypeDetail::NonNull(_) => __TypeKind::NonNull,
            TypeDetail::List(_) => __TypeKind::List,
        }
    }

    #[inline]
    async fn name(&self) -> Option<&str> {
        match &self.detail {
            TypeDetail::Named(ty) => Some(ty.name()),
            TypeDetail::NonNull(_) => None,
            TypeDetail::List(_) => None,
        }
    }

    #[inline]
    async fn description(&self) -> Option<&str> {
        match &self.detail {
            TypeDetail::Named(ty) => match ty {
                registry_v2::MetaType::Scalar(inner) => inner.description(),
                registry_v2::MetaType::Object(inner) => inner.description(),
                registry_v2::MetaType::Interface(inner) => inner.description(),
                registry_v2::MetaType::Union(inner) => inner.description(),
                registry_v2::MetaType::Enum(inner) => inner.description(),
                registry_v2::MetaType::InputObject(inner) => inner.description(),
            },
            TypeDetail::NonNull(_) => None,
            TypeDetail::List(_) => None,
        }
    }

    async fn fields(
        &self,
        _ctx: &ContextField<'_>,
        #[graphql(default = false)] include_deprecated: bool,
    ) -> Option<Vec<__Field<'a>>> {
        if let TypeDetail::Named(ty) = &self.detail {
            ty.fields().map(|fields| {
                fields
                    .filter(|field| (include_deprecated || !field.is_deprecated()) && !field.name().starts_with("__"))
                    .map(|field| __Field {
                        registry: self.registry,
                        field,
                    })
                    .collect()
            })
        } else {
            None
        }
    }

    async fn interfaces(&self) -> Option<Vec<__Type<'a>>> {
        if let TypeDetail::Named(registry_v2::MetaType::Object(object)) = &self.detail {
            Some(
                self.registry
                    .interfaces_implemented(object.name())
                    .map(|ty| __Type::new_simple(self.registry, ty))
                    .collect(),
            )
        } else {
            None
        }
    }

    async fn possible_types(&self) -> Option<Vec<__Type<'a>>> {
        match self.detail {
            TypeDetail::Named(registry_v2::MetaType::Interface(inner)) => Some(
                inner
                    .possible_types()
                    .map(|ty| __Type::new_simple(self.registry, ty))
                    .collect(),
            ),
            TypeDetail::Named(registry_v2::MetaType::Union(inner)) => Some(
                inner
                    .possible_types()
                    .map(|ty| __Type::new_simple(self.registry, ty))
                    .collect(),
            ),
            _ => None,
        }
    }

    async fn enum_values(
        &self,
        _ctx: &ContextField<'_>,
        #[graphql(default = false)] include_deprecated: bool,
    ) -> Option<Vec<__EnumValue<'a>>> {
        if let TypeDetail::Named(registry_v2::MetaType::Enum(enum_type)) = &self.detail {
            Some(
                enum_type
                    .values()
                    .filter(|value| include_deprecated || !value.is_deprecated())
                    .map(|value| __EnumValue {
                        registry: self.registry,
                        value,
                    })
                    .collect(),
            )
        } else {
            None
        }
    }

    async fn input_fields(&self, _ctx: &ContextField<'_>) -> Option<Vec<__InputValue<'a>>> {
        if let TypeDetail::Named(registry_v2::MetaType::InputObject(input_object)) = &self.detail {
            Some(
                input_object
                    .input_fields()
                    .map(|input_value| __InputValue {
                        registry: self.registry,
                        input_value,
                    })
                    .collect(),
            )
        } else {
            None
        }
    }

    #[inline]
    async fn of_type(&self) -> Option<__Type<'a>> {
        if let TypeDetail::List(ty) = &self.detail {
            Some(__Type::new(self.registry, ty))
        } else if let TypeDetail::NonNull(ty) = &self.detail {
            Some(__Type::new(self.registry, ty))
        } else {
            None
        }
    }

    #[graphql(name = "specifiedByURL")]
    async fn specified_by_url(&self) -> Option<&'a str> {
        if let TypeDetail::Named(registry_v2::MetaType::Scalar(scalar)) = &self.detail {
            scalar.specified_by_url()
        } else {
            None
        }
    }

    async fn is_one_of(&self) -> Option<bool> {
        if let TypeDetail::Named(registry_v2::MetaType::InputObject(input_object)) = &self.detail {
            Some(input_object.oneof())
        } else {
            None
        }
    }
}
