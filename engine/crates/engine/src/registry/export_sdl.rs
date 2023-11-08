use std::fmt::Write;

use super::{Deprecation, EnumType, InputObjectType, InterfaceType, ObjectType, ScalarType, UnionType};
use crate::registry::{MetaField, MetaInputValue, MetaType, Registry};

impl Registry {
    pub fn export_sdl(&self, federation: bool) -> String {
        let mut sdl = String::new();

        for ty in self.types.values() {
            if ty.name().starts_with("__") {
                continue;
            }

            if federation {
                const FEDERATION_TYPES: &[&str] = &["_Any", "_Entity", "_Service"];
                if FEDERATION_TYPES.contains(&ty.name()) {
                    continue;
                }
            }

            self.export_type(ty, &mut sdl, federation);
        }

        if !federation {
            writeln!(sdl, "schema {{").ok();
            writeln!(sdl, "\tquery: {}", self.query_type).ok();
            if let Some(mutation_type) = self.mutation_type.as_deref() {
                writeln!(sdl, "\tmutation: {mutation_type}").ok();
            }
            if let Some(subscription_type) = self.subscription_type.as_deref() {
                writeln!(sdl, "\tsubscription: {subscription_type}").ok();
            }
            writeln!(sdl, "}}").ok();
        }

        sdl
    }

    fn export_fields<'a, I: Iterator<Item = &'a MetaField>>(sdl: &mut String, it: I, federation: bool) {
        for field in it {
            if field.name.starts_with("__") || (federation && matches!(&*field.name, "_service" | "_entities")) {
                continue;
            }

            if field.description.is_some() {
                writeln!(
                    sdl,
                    "\t\"\"\"\n\t{}\n\t\"\"\"",
                    field.description.as_deref().unwrap().replace('\n', "\n\t")
                )
                .ok();
            }
            if !field.args.is_empty() {
                write!(sdl, "\t{}(", field.name).ok();
                for (i, arg) in field.args.values().enumerate() {
                    if i != 0 {
                        sdl.push_str(", ");
                    }
                    sdl.push_str(&export_input_value(arg));
                }
                write!(sdl, "): {}", field.ty).ok();
            } else {
                write!(sdl, "\t{}: {}", field.name, field.ty).ok();
            }

            if let Deprecation::Deprecated { reason } = &field.deprecation {
                write!(sdl, " @deprecated").ok();
                if let Some(reason) = reason {
                    write!(sdl, "(reason: \"{}\")", reason.escape_default()).ok();
                }
            }

            if federation {
                if field.external {
                    write!(sdl, " @external").ok();
                }
                if field.shareable {
                    write!(sdl, " @shareable").ok();
                }
                if let Some(from) = &field.r#override {
                    write!(sdl, " @override(from: \"{from}\")").ok();
                }
                if let Some(requires) = &field.requires {
                    write!(sdl, " @requires(fields: \"{requires}\")").ok();
                }
                if let Some(provides) = field.provides.as_deref() {
                    write!(sdl, " @provides(fields: \"{provides}\")").ok();
                }
                if field.inaccessible {
                    write!(sdl, " @inaccessible").ok();
                }
                for tag in &field.tags {
                    write!(sdl, " @tag(name: \"{}\")", tag.escape_default()).ok();
                }
            }

            writeln!(sdl).ok();
        }
    }

    fn export_type(&self, ty: &MetaType, sdl: &mut String, federation: bool) {
        match ty {
            MetaType::Scalar(ScalarType { name, description, .. }) => {
                const SYSTEM_SCALARS: &[&str] = &["Int", "Float", "String", "Boolean", "ID"];
                const FEDERATION_SCALARS: &[&str] = &["Any"];
                let mut export_scalar = !SYSTEM_SCALARS.contains(&name.as_str());
                if federation && FEDERATION_SCALARS.contains(&name.as_str()) {
                    export_scalar = false;
                }
                if export_scalar {
                    if description.is_some() {
                        writeln!(sdl, "\"\"\"\n{}\n\"\"\"", description.as_deref().unwrap()).ok();
                    }
                    writeln!(sdl, "scalar {name}").ok();
                }
            }
            MetaType::Object(ObjectType {
                name,
                fields,
                extends,
                description,
                external,
                shareable,
                ..
            }) => {
                if Some(name.as_str()) == self.subscription_type.as_deref()
                    && federation
                    && !self.federation_subscription
                {
                    return;
                }

                if name.as_str() == self.query_type && federation {
                    let mut field_count = 0;
                    for field in fields.values() {
                        if field.name.starts_with("__")
                            || (federation && matches!(&*field.name, "_service" | "_entities"))
                        {
                            continue;
                        }
                        field_count += 1;
                    }
                    if field_count == 0 {
                        return;
                    }
                }

                if description.is_some() {
                    writeln!(sdl, "\"\"\"\n{}\n\"\"\"", description.as_deref().unwrap()).ok();
                }
                if federation && *extends {
                    write!(sdl, "extend ").ok();
                }
                write!(sdl, "type {name} ").ok();
                self.write_implements(sdl, name);

                if federation {
                    if let Some(entity) = self.federation_entities.get(name) {
                        for key in entity.keys() {
                            let resolvable = if key.is_resolvable() { "" } else { " resolvable: false" };
                            write!(sdl, "@key(fields: \"{key}\"{resolvable}) ").ok();
                        }
                    }
                    if *external {
                        write!(sdl, "@external ").ok();
                    }
                    if *shareable {
                        write!(sdl, "@shareable ").ok();
                    }
                }

                writeln!(sdl, "{{").ok();
                Self::export_fields(sdl, fields.values(), federation);
                writeln!(sdl, "}}").ok();
            }
            MetaType::Interface(InterfaceType {
                name,
                fields,
                extends,
                description,
                ..
            }) => {
                if description.is_some() {
                    writeln!(sdl, "\"\"\"\n{}\n\"\"\"", description.as_deref().unwrap()).ok();
                }
                if federation && *extends {
                    write!(sdl, "extend ").ok();
                }
                write!(sdl, "interface {name} ").ok();
                if federation {
                    if let Some(entity) = self.federation_entities.get(name) {
                        for key in entity.keys() {
                            let resolvable = if key.is_resolvable() { "" } else { " resolvable: false" };
                            write!(sdl, "@key(fields: \"{key}\"{resolvable}) ").ok();
                        }
                    }
                }
                self.write_implements(sdl, name);

                writeln!(sdl, "{{").ok();
                Self::export_fields(sdl, fields.values(), federation);
                writeln!(sdl, "}}").ok();
            }
            MetaType::Enum(EnumType {
                name,
                enum_values,
                description,
                ..
            }) => {
                if description.is_some() {
                    writeln!(sdl, "\"\"\"\n{}\n\"\"\"", description.as_deref().unwrap()).ok();
                }
                write!(sdl, "enum {name} ").ok();
                writeln!(sdl, "{{").ok();
                for value in enum_values.values() {
                    write!(sdl, "\t{}", value.name).ok();

                    if let Deprecation::Deprecated { reason } = &value.deprecation {
                        write!(sdl, " @deprecated").ok();
                        if let Some(reason) = reason {
                            write!(sdl, "(reason: \"{}\")", reason.escape_default()).ok();
                        }
                    }
                    writeln!(sdl).ok();
                }
                writeln!(sdl, "}}").ok();
            }
            MetaType::InputObject(InputObjectType {
                name,
                input_fields,
                description,
                ..
            }) => {
                if description.is_some() {
                    writeln!(sdl, "\"\"\"\n{}\n\"\"\"", description.as_deref().unwrap()).ok();
                }
                write!(sdl, "input {name} ").ok();
                writeln!(sdl, "{{").ok();
                for field in input_fields.values() {
                    if let Some(description) = field.description.as_deref() {
                        writeln!(sdl, "\t\"\"\"\n\t{description}\n\t\"\"\"").ok();
                    }
                    writeln!(sdl, "\t{}", export_input_value(&field)).ok();
                }
                writeln!(sdl, "}}").ok();
            }
            MetaType::Union(UnionType {
                name,
                possible_types,
                description,
                ..
            }) => {
                if description.is_some() {
                    writeln!(sdl, "\"\"\"\n{}\n\"\"\"", description.as_deref().unwrap()).ok();
                }
                write!(sdl, "union {name} =").ok();
                for (idx, ty) in possible_types.iter().enumerate() {
                    if idx == 0 {
                        write!(sdl, " {ty}").ok();
                    } else {
                        write!(sdl, " | {ty}").ok();
                    }
                }
                writeln!(sdl).ok();
            }
        }
    }

    fn write_implements(&self, sdl: &mut String, name: &str) {
        if let Some(implements) = self.implements.get(name) {
            if !implements.is_empty() {
                write!(
                    sdl,
                    "implements {} ",
                    implements.iter().map(AsRef::as_ref).collect::<Vec<&str>>().join(" & ")
                )
                .ok();
            }
        }
    }
}

fn export_input_value(input_value: &MetaInputValue) -> String {
    if let Some(default_value) = &input_value.default_value {
        format!("{}: {} = {default_value}", input_value.name, input_value.ty)
    } else {
        format!("{}: {}", input_value.name, input_value.ty)
    }
}
