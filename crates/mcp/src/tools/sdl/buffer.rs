use super::WriteOptions;
use engine::Schema;
use engine_schema::{
    EnumDefinition, FieldDefinition, InputObjectDefinition, InputValueDefinition, InterfaceDefinition,
    ObjectDefinition, ScalarDefinition, TypeDefinition, UnionDefinition,
};
use itertools::Itertools;
use std::fmt::Write;

pub struct Buffer<'a> {
    schema: &'a Schema,
    buf: String,
    indent: usize,
}

impl std::fmt::Write for Buffer<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buf.push_str(s);
        Ok(())
    }
}

impl<'a> Buffer<'a> {
    pub fn new(schema: &'a Schema) -> Self {
        Self {
            schema,
            buf: String::with_capacity(1024),
            indent: 0,
        }
    }

    pub fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.buf.push_str("  ");
        }
    }

    pub fn write_quoted(&mut self, s: &str) {
        self.buf.push('"');
        for c in s.chars() {
            match c {
                '\r' => self.buf.push_str("\\r"),
                '\n' => self.buf.push_str("\\n"),
                '\t' => self.buf.push_str("\\t"),
                '"' => self.buf.push_str("\\\""),
                '\\' => self.buf.push_str("\\\\"),
                c if c.is_control() => write!(self.buf, "\\u{:04}", c as u32).unwrap(),
                c => self.buf.push(c),
            };
        }
        self.buf.push('"')
    }

    pub fn write_description(&mut self, description: &str) {
        self.write_indent();
        if description.contains('\n') {
            self.buf.push_str("\"\"\"\n");
            for line in description.split('\n') {
                self.write_indent();
                // Trim trailing whitespace
                let trimmed = line.trim_end();
                if !trimmed.is_empty() {
                    for part in Itertools::intersperse(trimmed.split('"'), r#"\""#) {
                        self.buf.push_str(part);
                    }
                }
                self.buf.push('\n');
            }
            self.write_indent();
            self.buf.push_str("\"\"\"\n");
        } else {
            self.buf.push('"');
            for part in Itertools::intersperse(description.trim_end().split('"'), r#"\""#) {
                self.buf.push_str(part);
            }
            self.buf.push_str("\"\n");
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn push(&mut self, c: char) {
        self.buf.push(c);
    }

    pub fn push_str(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    pub fn pop(&mut self) -> Option<char> {
        self.buf.pop()
    }

    pub fn into_string(self) -> String {
        self.buf
    }

    pub fn write_input_value(&mut self, input_value: InputValueDefinition<'_>, indent: bool) {
        if let Some(description) = input_value.description() {
            self.write_description(description);
        }

        if indent {
            self.write_indent()
        }
        self.push_str(input_value.name());
        self.push_str(": ");
        write!(self, "{}", input_value.ty()).unwrap();
        if let Some(default_value) = input_value.default_value() {
            self.push_str(" = ");
            write!(self, "{}", default_value).unwrap();
        }
        self.push('\n');
    }

    pub fn write_field_definition(&mut self, field: FieldDefinition<'_>) {
        if let Some(description) = field.description() {
            self.write_description(description);
        }

        self.write_indent();
        self.push_str(field.name());

        // Format arguments if any
        let args = field.arguments();
        if args.len() > 0 {
            self.push('(');
            self.indent += 1;

            let multiline = field.arguments().any(|arg| arg.description().is_some());
            if multiline {
                self.push('\n');
                for (i, arg) in args.enumerate() {
                    if i > 0 {
                        let last = self.pop();
                        debug_assert_eq!(last, Some('\n'));
                        self.push_str(",\n");
                    }
                    self.write_input_value(arg, true);
                }
                self.indent -= 1;
                self.write_indent();
            } else {
                for (i, arg) in args.enumerate() {
                    if i > 0 {
                        let last = self.pop();
                        debug_assert_eq!(last, Some('\n'));
                        self.push_str(", ");
                    }
                    self.write_input_value(arg, false);
                }
                self.indent -= 1;
                let last = self.pop();
                debug_assert_eq!(last, Some('\n'));
            }

            self.push(')');
        }

        write!(self, ": {}", field.ty()).unwrap();
        if field.has_deprecated().is_some() {
            self.push_str(" @deprecated");
        }
        self.push('\n');
    }

    pub fn write_object_definition(&mut self, object: ObjectDefinition<'_>, opt: &WriteOptions) {
        if let Some(description) = object.description() {
            self.write_description(description);
        }

        if opt.fields_subset.is_some() {
            self.push_str("# Incomplete fields\n");
        }
        self.push_str("type ");
        self.push_str(object.name());

        if opt.interfaces && object.interfaces().len() > 0 {
            self.push_str(" implements ");
            for (i, interface) in object.interfaces().enumerate() {
                if i > 0 {
                    self.push_str(" & ");
                }
                self.push_str(interface.name());
            }
        }

        if object.has_deprecated().is_some() {
            self.push_str(" @deprecated");
        }

        self.push_str(" {\n");
        self.indent += 1;

        match &opt.fields_subset {
            Some(field_ids) => {
                for field_id in field_ids {
                    self.write_field_definition(self.schema.walk(field_id));
                }
            }
            None => {
                for field in object.fields() {
                    self.write_field_definition(field);
                }
            }
        }

        self.indent -= 1;
        self.write_indent();
        self.push_str("}\n\n");
    }

    pub fn write_interface_definition(&mut self, interface: InterfaceDefinition<'_>, opt: &WriteOptions) {
        if let Some(description) = interface.description() {
            self.write_description(description);
        }

        if opt.fields_subset.is_some() {
            self.push_str("# Incomplete fields\n");
        }
        self.push_str("interface ");
        self.push_str(interface.name());

        if opt.interfaces && interface.interfaces().len() > 0 {
            self.push_str(" implements ");
            for (i, interface) in interface.interfaces().enumerate() {
                if i > 0 {
                    self.push_str(" & ");
                }
                self.push_str(interface.name());
            }
        }

        self.push_str(" {\n");
        self.indent += 1;

        match &opt.fields_subset {
            Some(field_ids) => {
                for field_id in field_ids {
                    self.write_field_definition(self.schema.walk(field_id));
                }
            }
            None => {
                for field in interface.fields() {
                    self.write_field_definition(field);
                }
            }
        }

        self.indent -= 1;
        self.write_indent();
        self.push_str("}\n\n");
    }

    pub fn write_scalar_definition(&mut self, scalar: ScalarDefinition<'_>) {
        if matches!(scalar.name(), "String" | "Boolean" | "Int" | "Float" | "ID" | "JSON") {
            return;
        }

        if let Some(description) = scalar.description() {
            self.write_description(description);
        }

        self.push_str("scalar ");
        self.push_str(scalar.name());

        if let Some(url) = scalar.specified_by_url() {
            self.push_str(" @specifiedBy(url: ");
            self.write_quoted(url);
            self.push(')');
        }

        self.push_str("\n\n");
    }

    pub fn write_enum_definition(&mut self, enum_def: EnumDefinition<'_>) {
        if let Some(description) = enum_def.description() {
            self.write_description(description);
        }

        self.push_str("enum ");
        self.push_str(enum_def.name());
        self.push_str(" {\n");
        self.indent += 1;

        for value in enum_def.values() {
            if let Some(description) = value.description() {
                self.write_description(description);
            }
            self.write_indent();
            self.push_str(value.name());
            if value.has_deprecated().is_some() {
                self.push_str(" @deprecated");
            }
            self.push('\n');
        }

        self.indent -= 1;
        self.write_indent();
        self.push_str("}\n\n");
    }

    pub fn write_union_definition(&mut self, union: UnionDefinition<'_>) {
        if let Some(description) = union.description() {
            self.write_description(description);
        }

        self.push_str("union ");
        self.push_str(union.name());
        self.push_str(" = ");

        let types: Vec<_> = union.possible_types_ordered_by_typename().collect();
        for (i, ty) in types.iter().enumerate() {
            if i > 0 {
                self.push_str(" | ");
            }
            self.push_str(ty.name());
        }

        self.push_str("\n\n");
    }

    pub fn write_input_object_definition(&mut self, input_object: InputObjectDefinition<'_>) {
        if let Some(description) = input_object.description() {
            self.write_description(description);
        }

        self.push_str("input ");
        self.push_str(input_object.name());
        self.push_str(" {\n");
        self.indent += 1;

        for field in input_object.input_fields() {
            self.write_input_value(field, true);
        }

        self.indent -= 1;
        self.write_indent();
        self.push_str("}\n\n");
    }

    pub fn write_type_definition(&mut self, type_def: TypeDefinition<'_>, opt: &WriteOptions) {
        match type_def {
            TypeDefinition::Scalar(def) => self.write_scalar_definition(def),
            TypeDefinition::Object(def) => self.write_object_definition(def, opt),
            TypeDefinition::Interface(def) => self.write_interface_definition(def, opt),
            TypeDefinition::Union(def) => self.write_union_definition(def),
            TypeDefinition::Enum(def) => self.write_enum_definition(def),
            TypeDefinition::InputObject(def) => self.write_input_object_definition(def),
        }
    }
}
