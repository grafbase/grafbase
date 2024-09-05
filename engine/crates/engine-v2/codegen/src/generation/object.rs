mod debug;
mod reader;
mod struct_;

use std::{borrow::Cow, collections::HashSet};

use tracing::instrument;

use crate::domain::{Definition, Domain, Field, Object, StorageType, Union, UnionKind};

use super::{id::generate_id, GeneratedCode, Imports};

#[instrument(skip(domain))]
pub fn generate_object<'a>(domain: &'a Domain, object: &'a Object) -> anyhow::Result<GeneratedCode<'a>> {
    let mut imported_definition_names = HashSet::new();

    let fields = object
        .fields
        .iter()
        .map(|field| -> anyhow::Result<FieldContext> {
            imported_definition_names.insert(field.type_name.as_str());
            Ok(FieldContext {
                domain,
                field,
                ty: domain
                    .definitions_by_name
                    .get(&field.type_name)
                    .ok_or_else(|| anyhow::anyhow!("Could not find type {}", field.type_name))?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let mut code_sections = struct_::generate_struct(domain, object, &fields)?;

    if let Some(indexed) = &object.indexed {
        code_sections.extend(generate_id(domain, indexed)?);
    }
    code_sections.extend(reader::generate_reader(domain, object, &fields)?);

    Ok(GeneratedCode {
        module_path: &object.meta.module_path,
        code_sections,
        imports: Imports {
            generated: imported_definition_names,
            readable: if object.fields.iter().any(|field| field.has_list_wrapping()) {
                HashSet::from_iter(["Iter"])
            } else {
                Default::default()
            },
        },
    })
}

#[derive(Clone, Copy)]
pub struct FieldContext<'a> {
    domain: &'a Domain,
    field: &'a Field,
    ty: &'a Definition,
}

impl<'a> std::ops::Deref for FieldContext<'a> {
    type Target = Field;
    fn deref(&self) -> &Self::Target {
        self.field
    }
}

impl FieldContext<'_> {
    pub fn struct_field_name(&self) -> Cow<'_, str> {
        let name = &self.name;
        if matches!(self.ty.storage_type(), StorageType::Id { .. })
            || matches!(
                self.ty,
                Definition::Union(Union {
                    kind: UnionKind::Id(_),
                    ..
                })
            )
        {
            if self.has_list_wrapping() {
                let name = name.strip_suffix("s").unwrap_or(name);
                Cow::Owned(format!("{name}_ids"))
            } else {
                Cow::Owned(format!("{name}_id"))
            }
        } else {
            name.into()
        }
    }

    pub fn reader_method_name(&self) -> Cow<'_, str> {
        (&self.name).into()
    }
}
