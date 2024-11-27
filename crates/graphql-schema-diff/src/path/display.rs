use super::*;

impl fmt::Display for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Path::SchemaDefinition => f.write_str(":schema"),
            Path::SchemaExtension(idx) => {
                f.write_str(":schema")?;
                write_index(idx, f)
            }
            Path::TypeDefinition(type_name, path_in_type) => {
                f.write_str(type_name)?;

                if let Some(path_in_type) = path_in_type {
                    f.write_str(".")?;
                    path_in_type.fmt(f)
                } else {
                    Ok(())
                }
            }
            Path::TypeExtension(type_name, extension_idx, path_in_type) => {
                f.write_str(type_name)?;
                write_index(extension_idx, f)?;

                if let Some(path_in_type) = path_in_type {
                    f.write_str(".")?;
                    path_in_type.fmt(f)
                } else {
                    Ok(())
                }
            }
            Path::DirectiveDefinition(directive_name) => {
                f.write_str("@")?;
                f.write_str(directive_name)
            }
        }
    }
}

impl fmt::Display for PathInType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathInType::InField(field_name, path_in_field) => {
                f.write_str(field_name)?;

                if let Some(path_in_field) = path_in_field {
                    f.write_str(".")?;
                    path_in_field.fmt(f)
                } else {
                    Ok(())
                }
            }
            PathInType::InDirective(directive_name, directive_index) => {
                f.write_str("@")?;
                f.write_str(directive_name)?;
                write_index(directive_index, f)
            }
            PathInType::InterfaceImplementation(interface_name) => {
                f.write_str("&")?;
                f.write_str(interface_name)
            }
        }
    }
}

impl fmt::Display for PathInField<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathInField::InArgument(arg_name) => f.write_str(arg_name),
            PathInField::InDirective(directive_name, directive_idx) => {
                f.write_str("@")?;
                f.write_str(directive_name)?;
                write_index(directive_idx, f)
            }
        }
    }
}

fn write_index(idx: &usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("[")?;
    fmt::Display::fmt(&idx, f)?;
    f.write_str("]")
}
