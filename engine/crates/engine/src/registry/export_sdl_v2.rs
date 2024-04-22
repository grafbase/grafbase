use std::fmt::Write;

// use super::{Deprecation, EnumType, InputObjectType, InterfaceType, ObjectType, ScalarType, UnionType};
use registry_v2::{Deprecation, MetaField, MetaInputValue, MetaType, Registry};

use super::field_set::FieldSetDisplay;

pub trait RegistrySdlExt {
    fn export_sdl(&self, federation: bool) -> String;
}

impl RegistrySdlExt for Registry {
    fn export_sdl(&self, federation: bool) -> String {
        let mut sdl = String::new();

        if federation {
            writeln!(sdl, "extend schema @link(").ok();
            writeln!(sdl, "\turl: \"https://specs.apollo.dev/federation/v2.3\",").ok();
            writeln!(sdl, "\timport: [\"@key\", \"@tag\", \"@shareable\", \"@inaccessible\", \"@override\", \"@external\", \"@provides\", \"@requires\", \"@composeDirective\", \"@interfaceObject\"]").ok();
            writeln!(sdl, ")").ok();
        }

        for ty in self.types() {
            if ty.name().starts_with("__") {
                continue;
            }

            if federation {
                const FEDERATION_TYPES: &[&str] = &["_Any", "_Entity", "_Service"];
                if FEDERATION_TYPES.contains(&ty.name()) {
                    continue;
                }
            }

            export_type(self, ty, &mut sdl, federation);
        }

        if !federation {
            writeln!(sdl, "schema {{").ok();
            writeln!(sdl, "\tquery: {}", self.query_type().name()).ok();
            if let Some(mutation_type) = self.mutation_type() {
                writeln!(sdl, "\tmutation: {}", mutation_type.name()).ok();
            }
            if let Some(subscription_type) = self.subscription_type() {
                writeln!(sdl, "\tsubscription: {}", subscription_type.name()).ok();
            }
            writeln!(sdl, "}}").ok();
        }

        sdl
    }
}

fn export_fields<'a, I: Iterator<Item = MetaField<'a>>>(sdl: &mut String, it: I, federation: bool) {
    for field in it {
        if field.name().starts_with("__") || (federation && matches!(field.name(), "_service" | "_entities")) {
            continue;
        }

        if let Some(description) = field.description() {
            writeln!(sdl, "\t\"\"\"\n\t{}\n\t\"\"\"", description.replace('\n', "\n\t")).ok();
        }
        let args = field.args();
        if args.len() != 0 {
            write!(sdl, "\t{}(", field.name()).ok();
            for (i, arg) in args.enumerate() {
                if i != 0 {
                    sdl.push_str(", ");
                }
                sdl.push_str(&export_input_value(arg));
            }
            write!(sdl, "): {}", field.ty().to_string()).ok();
        } else {
            write!(sdl, "\t{}: {}", field.name(), field.ty().to_string()).ok();
        }

        if let Some(Deprecation::Deprecated { reason }) = field.deprecation() {
            write!(sdl, " @deprecated").ok();
            if let Some(reason) = reason {
                write!(sdl, "(reason: \"{}\")", reason.escape_default()).ok();
            }
        }

        if federation {
            if let Some(federation_field) = field.federation() {
                if federation_field.external {
                    write!(sdl, " @external").ok();
                }
                if federation_field.shareable {
                    write!(sdl, " @shareable").ok();
                }
                if let Some(from) = &federation_field.r#override {
                    write!(sdl, " @override(from: \"{from}\")").ok();
                }
                if let Some(provides) = federation_field.provides.as_deref() {
                    write!(sdl, " @provides(fields: \"{provides}\")").ok();
                }
                if federation_field.inaccessible {
                    write!(sdl, " @inaccessible").ok();
                }
                for tag in &federation_field.tags {
                    write!(sdl, " @tag(name: \"{}\")", tag.escape_default()).ok();
                }
            }
            if let Some(requires) = field.requires() {
                write!(sdl, " @requires(fields: \"{}\")", FieldSetDisplay(requires)).ok();
            }
        }

        writeln!(sdl).ok();
    }
}

fn export_type(registry: &registry_v2::Registry, ty: MetaType<'_>, sdl: &mut String, federation: bool) {
    let extends = false; // TODO: Reintroduce this if neccesary
    match ty {
        MetaType::Scalar(scalar) => {
            const SYSTEM_SCALARS: &[&str] = &["Int", "Float", "String", "Boolean", "ID"];
            const FEDERATION_SCALARS: &[&str] = &["Any"];
            let name = scalar.name();
            let mut export_scalar = !SYSTEM_SCALARS.contains(&name);
            if federation && FEDERATION_SCALARS.contains(&name) {
                export_scalar = false;
            }
            if export_scalar {
                if let Some(description) = scalar.description() {
                    writeln!(sdl, "\"\"\"\n{description}\n\"\"\"").ok();
                }
                writeln!(sdl, "scalar {name}").ok();
            }
        }
        MetaType::Object(object) => {
            let name = object.name();
            if Some(name) == registry.subscription_type().map(|ty| ty.name())
                && federation
                && !registry.federation_subscription
            {
                return;
            }

            if name == registry.query_type().name() && federation {
                let mut field_count = 0;
                for field in object.fields() {
                    if field.name().starts_with("__")
                        || (federation && matches!(field.name(), "_service" | "_entities"))
                    {
                        continue;
                    }
                    field_count += 1;
                }
                if field_count == 0 {
                    return;
                }
            }

            if let Some(description) = object.description() {
                writeln!(sdl, "\"\"\"\n{description}\n\"\"\"").ok();
            }
            if federation && extends {
                write!(sdl, "extend ").ok();
            }
            write!(sdl, "type {name} ").ok();
            write_implements(registry, sdl, name);

            if federation {
                if let Some(entity) = registry.federation_entities.get(name) {
                    for key in entity.keys.iter() {
                        let resolvable = if key.is_resolvable() { "" } else { " resolvable: false" };
                        write!(
                            sdl,
                            "@key(fields: \"{}\"{resolvable}) ",
                            FieldSetDisplay(&key.selections)
                        )
                        .ok();
                    }
                }
                if object.external() {
                    write!(sdl, "@external ").ok();
                }
                if object.shareable() {
                    write!(sdl, "@shareable ").ok();
                }
            }

            writeln!(sdl, "{{").ok();
            export_fields(sdl, object.fields(), federation);
            writeln!(sdl, "}}").ok();
        }
        MetaType::Interface(interface) => {
            let name = interface.name();
            if let Some(description) = interface.description() {
                writeln!(sdl, "\"\"\"\n{description}\n\"\"\"",).ok();
            }
            if federation && extends {
                write!(sdl, "extend ").ok();
            }
            write!(sdl, "interface {} ", interface.name()).ok();
            if federation {
                if let Some(entity) = registry.federation_entities.get(name) {
                    for key in entity.keys.iter() {
                        let resolvable = if key.is_resolvable() { "" } else { " resolvable: false" };
                        write!(
                            sdl,
                            "@key(fields: \"{}\"{resolvable}) ",
                            FieldSetDisplay(&key.selections)
                        )
                        .ok();
                    }
                }
            }
            write_implements(registry, sdl, name);

            writeln!(sdl, "{{").ok();
            export_fields(sdl, interface.fields(), federation);
            writeln!(sdl, "}}").ok();
        }
        MetaType::Enum(enum_type) => {
            if let Some(description) = enum_type.description() {
                writeln!(sdl, "\"\"\"\n{description}\n\"\"\"",).ok();
            }
            write!(sdl, "enum {} ", enum_type.name()).ok();
            writeln!(sdl, "{{").ok();
            for value in enum_type.values() {
                write!(sdl, "\t{}", value.name()).ok();

                if let Some(Deprecation::Deprecated { reason }) = &value.deprecation() {
                    write!(sdl, " @deprecated").ok();
                    if let Some(reason) = reason {
                        write!(sdl, "(reason: \"{}\")", reason.escape_default()).ok();
                    }
                }
                writeln!(sdl).ok();
            }
            writeln!(sdl, "}}").ok();
        }
        MetaType::InputObject(input_object) => {
            let name = input_object.name();
            if let Some(description) = input_object.description() {
                writeln!(sdl, "\"\"\"\n{description}\n\"\"\"",).ok();
            }
            write!(sdl, "input {name}").ok();

            let input_fields = input_object.input_fields();
            if input_fields.len() != 0 {
                writeln!(sdl, " {{").ok();
                for field in input_fields {
                    if let Some(description) = field.description() {
                        writeln!(sdl, "\t\"\"\"\n\t{description}\n\t\"\"\"").ok();
                    }
                    writeln!(sdl, "\t{}", export_input_value(field)).ok();
                }
                writeln!(sdl, "}}").ok();
            } else {
                writeln!(sdl).ok();
            }
        }
        MetaType::Union(union_type) => {
            let name = union_type.name();
            if let Some(description) = union_type.description() {
                writeln!(sdl, "\"\"\"\n{description}\n\"\"\"",).ok();
            }
            write!(sdl, "union {name} =").ok();
            for (idx, ty) in union_type.possible_types().enumerate() {
                if idx == 0 {
                    write!(sdl, " {}", ty.name()).ok();
                } else {
                    write!(sdl, " | {}", ty.name()).ok();
                }
            }
            writeln!(sdl).ok();
        }
    }
}

fn write_implements(registry: &Registry, sdl: &mut String, name: &str) {
    let implements = registry.interfaces_implemented(name);
    if implements.len() != 0 {
        write!(
            sdl,
            "implements {} ",
            implements.map(|ty| ty.name()).collect::<Vec<&str>>().join(" & ")
        )
        .ok();
    }
}

fn export_input_value(input_value: MetaInputValue) -> String {
    if let Some(default_value) = input_value.default_value() {
        format!(
            "{}: {} = {default_value}",
            input_value.name(),
            input_value.ty().to_string()
        )
    } else {
        format!("{}: {}", input_value.name(), input_value.ty().to_string())
    }
}
