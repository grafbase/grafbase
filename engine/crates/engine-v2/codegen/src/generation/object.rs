mod debug;
mod struct_;
mod walker;

use std::{borrow::Cow, collections::HashSet};

use tracing::instrument;

use crate::domain::{Definition, Domain, Field, Object};

use super::{id::generate_id, GeneratedCode, Imports};

#[instrument(skip(domain))]
pub fn generate_object<'a>(domain: &'a Domain, object: &'a Object) -> anyhow::Result<GeneratedCode<'a>> {
    let mut imported_definition_names = HashSet::new();

    let fields = object
        .fields
        .iter()
        .map(|field| -> anyhow::Result<FieldContext> {
            imported_definition_names.insert(field.type_name.as_str());
            let ty = domain
                .definitions_by_name
                .get(&field.type_name)
                .ok_or_else(|| anyhow::anyhow!("Could not find type {}", field.type_name))?;
            Ok(FieldContext {
                domain,
                field,
                // A walker is generated whenever the field name, as defined in the GraphQL SDL
                // isn't the same as the struct field name. This only happens if we have a proper
                // Walker type do not return a simple ref.
                has_walker: field.record_field_name != field.name,
                ty,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let mut code_sections = struct_::generate_struct(domain, object, &fields)?;

    if let Some(indexed) = &object.indexed {
        code_sections.extend(generate_id(domain, indexed)?);
    }
    code_sections.extend(walker::generate_walker(domain, object, &fields)?);

    Ok(GeneratedCode {
        module_path: &object.meta.module_path,
        code_sections,
        imports: Imports {
            generated: imported_definition_names,
            walker_lib: if object.fields.iter().any(|field| field.has_list_wrapping()) {
                HashSet::from_iter(["Iter"])
            } else {
                Default::default()
            },
        },
    })
}

pub struct FieldContext<'a> {
    domain: &'a Domain,
    has_walker: bool,
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
    pub fn walker_method_name(&self) -> Cow<'_, str> {
        (&self.name).into()
    }
}
