//! Additional functionality for generated types

use crate::{
    ids::{MetaEnumValueId, MetaFieldId, MetaInputValueId},
    EnumType, InputObjectType, InterfaceType, Iter, MetaDirective, MetaEnumValue, MetaField, MetaInputValue, MetaType,
    ObjectType, RecordLookup, UnionType,
};

impl<'a> MetaType<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            MetaType::Object(inner) => inner.name(),
            MetaType::Interface(inner) => inner.name(),
            MetaType::Union(inner) => inner.name(),
            MetaType::Enum(inner) => inner.name(),
            MetaType::InputObject(inner) => inner.name(),
            MetaType::Scalar(inner) => inner.name(),
        }
    }

    pub fn description(&self) -> Option<&'a str> {
        match self {
            MetaType::Object(inner) => inner.description(),
            MetaType::Interface(inner) => inner.description(),
            MetaType::Union(inner) => inner.description(),
            MetaType::Enum(inner) => inner.description(),
            MetaType::InputObject(inner) => inner.description(),
            MetaType::Scalar(inner) => inner.description(),
        }
    }

    pub fn fields(&self) -> Option<Iter<'a, MetaField<'a>>> {
        match self {
            MetaType::Object(inner) => Some(inner.fields()),
            MetaType::Interface(inner) => Some(inner.fields()),
            _ => None,
        }
    }

    pub fn as_input_object(&self) -> Option<InputObjectType<'a>> {
        match self {
            MetaType::InputObject(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn is_composite(&self) -> bool {
        matches!(self, MetaType::Object(_) | MetaType::Interface(_) | MetaType::Union(_))
    }

    pub fn is_abstract(&self) -> bool {
        matches!(self, MetaType::Interface(_) | MetaType::Union(_))
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, MetaType::Enum(_) | MetaType::Scalar(_))
    }

    pub fn is_input(&self) -> bool {
        matches!(self, MetaType::Enum(_) | MetaType::Scalar(_) | MetaType::InputObject(_))
    }

    pub fn possible_types(&self) -> Option<Box<dyn ExactSizeIterator<Item = MetaType<'a>> + 'a>> {
        match self {
            MetaType::Interface(iface) => Some(Box::new(iface.possible_types())),
            MetaType::Union(union) => Some(Box::new(union.possible_types())),
            _ => None,
        }
    }

    pub fn is_possible_type(&self, other: &str) -> bool {
        match self {
            MetaType::Interface(inner) => inner.possible_types().any(|ty| ty.name() == other),
            MetaType::Union(inner) => inner.possible_types().any(|ty| ty.name() == other),
            MetaType::Object(inner) => inner.name() == other,
            _ => false,
        }
    }
}

impl<'a> MetaType<'a> {
    pub fn field(&self, name: &str) -> Option<MetaField<'a>> {
        match self {
            MetaType::Object(obj) => obj.field(name),
            MetaType::Interface(iface) => iface.field(name),
            _ => None,
        }
    }
}

impl<'a> ObjectType<'a> {
    pub fn field(&self, name: &str) -> Option<MetaField<'a>> {
        if name == "__typename" {
            return Some(self.0.registry.read(self.0.registry.typename_index));
        }

        let object = self.0.registry.lookup(self.0.id);
        let index = self.0.registry.object_fields[object.fields.start.to_index()..object.fields.end.to_index()]
            .binary_search_by(|field| self.0.registry.string_cmp(field.name, name))
            .ok()?;

        Some(
            self.0
                .registry
                .read(MetaFieldId::new(object.fields.start.to_index() + index)),
        )
    }
}

impl<'a> MetaField<'a> {
    pub fn is_deprecated(&self) -> bool {
        self.deprecation()
            .map(|deprecation| deprecation.is_deprecated())
            .unwrap_or_default()
    }

    pub fn deprecation_reason(&self) -> Option<&'a str> {
        self.deprecation().and_then(|deprecation| deprecation.reason())
    }
}

impl<'a> InterfaceType<'a> {
    pub fn field(&self, name: &str) -> Option<MetaField<'a>> {
        if name == "__typename" {
            return Some(self.0.registry.read(self.0.registry.typename_index));
        }

        let object = self.0.registry.lookup(self.0.id);
        let index = self.0.registry.object_fields[object.fields.start.to_index()..object.fields.end.to_index()]
            .binary_search_by(|field| self.0.registry.string_cmp(field.name, name))
            .ok()?;

        Some(
            self.0
                .registry
                .read(MetaFieldId::new(object.fields.start.to_index() + index)),
        )
    }
}

impl<'a> UnionType<'a> {
    /// Unions don't really have fields, but we implement this just for __typename
    pub fn field(&self, name: &str) -> Option<MetaField<'a>> {
        if name == "__typename" {
            return Some(self.0.registry.read(self.0.registry.typename_index));
        }
        None
    }
}

impl<'a> EnumType<'a> {
    pub fn value(&self, name: &str) -> Option<MetaEnumValue<'a>> {
        let enum_type = self.0.registry.lookup(self.0.id);
        let index = self.0.registry.enum_values[enum_type.values.start.to_index()..enum_type.values.end.to_index()]
            .binary_search_by(|value| self.0.registry.string_cmp(value.name, name))
            .ok()?;

        Some(
            self.0
                .registry
                .read(MetaEnumValueId::new(enum_type.values.start.to_index() + index)),
        )
    }
}

impl<'a> MetaEnumValue<'a> {
    pub fn is_deprecated(&self) -> bool {
        self.deprecation()
            .map(|deprecation| deprecation.is_deprecated())
            .unwrap_or_default()
    }

    pub fn deprecation_reason(&self) -> Option<&'a str> {
        self.deprecation().and_then(|deprecation| deprecation.reason())
    }
}

impl<'a> InputObjectType<'a> {
    pub fn field(&self, name: &str) -> Option<MetaInputValue<'a>> {
        let object = self.0.registry.lookup(self.0.id);
        let index = self.0.registry.input_values
            [object.input_fields.start.to_index()..object.input_fields.end.to_index()]
            .binary_search_by(|field| self.0.registry.string_cmp(field.name, name))
            .ok()?;

        Some(
            self.0
                .registry
                .read(MetaInputValueId::new(object.input_fields.start.to_index() + index)),
        )
    }
}

impl<'a> MetaField<'a> {
    pub fn argument(&self, name: &str) -> Option<MetaInputValue<'a>> {
        let field = self.0.registry.lookup(self.0.id);
        let index = self.0.registry.input_values[field.args.start.to_index()..field.args.end.to_index()]
            .binary_search_by(|value| self.0.registry.string_cmp(value.name, name))
            .ok()?;

        Some(
            self.0
                .registry
                .read(MetaInputValueId::new(field.args.start.to_index() + index)),
        )
    }

    pub fn target_field_name(&self) -> &str {
        self.mapped_name().unwrap_or(self.name())
    }
}

impl<'a> MetaDirective<'a> {
    pub fn argument(&self, name: &str) -> Option<MetaInputValue<'a>> {
        let directive = self.0.registry.lookup(self.0.id);
        let index = self.0.registry.input_values[directive.args.start.to_index()..directive.args.end.to_index()]
            .binary_search_by(|value| self.0.registry.string_cmp(value.name, name))
            .ok()?;

        Some(
            self.0
                .registry
                .read(MetaInputValueId::new(directive.args.start.to_index() + index)),
        )
    }
}
