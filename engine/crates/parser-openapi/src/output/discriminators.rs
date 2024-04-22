use std::collections::HashSet;

use itertools::Itertools;
use registry_v2::UnionDiscriminator;

use crate::graph::{OpenApiGraph, ScalarKind};

impl crate::graph::OutputType {
    /// Tries to find discriminators for a union - fields we can use to tell which variant
    /// of the union we've received from the remote API.
    ///
    /// OpenAPI specifically provides a field to specify this, but nobody seems to use it.
    /// I've also read advice saying you should strive not to use it.  Apparently people
    /// want to make this as hard as they possibly can.
    ///
    /// The "proper" way to do this would be to run everything through a JSON schema validator,
    /// but pulling the entire specs JSON schema into the registry is likely to be problematic
    /// so this is a best effort attempt to avoid that.
    pub fn discriminators(self, graph: &OpenApiGraph) -> Vec<(String, registry_v2::UnionDiscriminator)> {
        let possible_types = self.possible_types(graph);

        // The easiest discriminator is a field that's unique to a given member on the union.
        // To figure this out we need to find all the names that appear > once.
        let unsuitable_names = possible_types
            .iter()
            .flat_map(|ty| {
                ty.fields(graph)
                    .into_iter()
                    .map(|field| field.openapi_name)
                    .collect::<Vec<_>>()
            })
            .duplicates()
            .collect::<HashSet<_>>();

        let mut discriminators = possible_types
            .iter()
            .map(|ty| {
                ty.scalar_wrapper_discriminator(graph)
                    .or_else(|| ty.unique_field_discriminator(&unsuitable_names, graph))
                    .or_else(|| ty.possible_value_discriminator(graph, &possible_types))
            })
            .zip(&possible_types)
            .collect::<Vec<_>>();

        // The above strategies aren't always good enough but we can support one single variant
        // without a discriminator as a fallback if none of the others match.
        // We sort by Some/None so these end up at the end of the list.
        discriminators.sort_by_key(|(discriminator, _)| !discriminator.is_some());

        if let Some((fallback, _)) = discriminators
            .iter_mut()
            .find(|(discriminator, _)| discriminator.is_none())
        {
            *fallback = Some(UnionDiscriminator::Fallback);
        }

        discriminators
            .into_iter()
            .filter_map(|(discriminator, ty)| {
                if discriminator.is_none() {
                    tracing::info!(
                        "Couldn't find a discriminator for {} in {}",
                        ty.name(graph).unwrap(),
                        self.name(graph).unwrap()
                    );
                }

                Some((ty.name(graph)?, discriminator?))
            })
            .collect()
    }

    fn scalar_wrapper_discriminator(self, graph: &OpenApiGraph) -> Option<UnionDiscriminator> {
        Some(UnionDiscriminator::IsAScalar(
            self.inner_scalar_kind(graph)?.try_into().ok()?,
        ))
    }

    /// Finds a required field thats name is unique to this member of a union, which
    /// we can use as a discriminator.
    fn unique_field_discriminator(
        self,
        unsuitable_names: &HashSet<String>,
        graph: &OpenApiGraph,
    ) -> Option<UnionDiscriminator> {
        self.fields(graph)
            .iter()
            .filter(|field| field.ty.is_required())
            .find_map(|field| {
                let name = &field.openapi_name;
                (!unsuitable_names.contains(name)).then(|| name.clone())
            })
            .map(UnionDiscriminator::FieldPresent)
    }

    // Tries to find a discriminator based on unique value(s) specified on
    // a given field
    //
    // To do this we need to find fields that have specific values set on them
    // and _either_ do not exist on the other types in the union _or_ have a
    // different set of values defined on all the other types
    fn possible_value_discriminator(
        self,
        graph: &OpenApiGraph,
        possible_types: &[crate::graph::OutputType],
    ) -> Option<UnionDiscriminator> {
        let value_discriminator: Option<crate::graph::OutputField> = self
            .fields(graph)
            .into_iter()
            .filter(|field| !field.ty.possible_values(graph).is_empty())
            .find(|field| {
                let possible_values = field
                    .ty
                    .possible_values(graph)
                    .into_iter()
                    .map(|value| value.to_string())
                    .collect::<HashSet<_>>();

                !possible_types
                    .iter()
                    .filter(|other_type| **other_type != self)
                    .filter_map(|other_type| {
                        // Find other_types equivalent to field if it exists
                        other_type.field(&field.openapi_name, graph)
                    })
                    .any(|sibling_field| {
                        let sibling_possible_values = sibling_field.ty.possible_values(graph);

                        // If this sibling has an equivalent field we need to make sure none of its values clash.
                        //
                        // If there are no possible values on a sibling then that sibling could be any
                        // value so we can't use this field to discriminate.
                        //
                        // Similarly, if a sibling uses any of the same values then it's a bit ambiguous.
                        sibling_possible_values.is_empty()
                            || sibling_possible_values
                                .into_iter()
                                .any(|sibling_value| possible_values.contains(&sibling_value.to_string()))
                    })
            });

        value_discriminator.map(|field| {
            UnionDiscriminator::FieldHasValue(
                field.openapi_name.clone(),
                field.ty.possible_values(graph).into_iter().cloned().collect(),
            )
        })
    }
}

impl TryFrom<ScalarKind> for registry_v2::ScalarKind {
    type Error = ();

    fn try_from(value: ScalarKind) -> Result<Self, Self::Error> {
        use registry_v2::ScalarKind as RegistryScalarKind;
        match value {
            ScalarKind::String => Ok(RegistryScalarKind::String),
            ScalarKind::Integer | ScalarKind::Float => Ok(RegistryScalarKind::Number),
            ScalarKind::Boolean => Ok(RegistryScalarKind::Boolean),
            ScalarKind::Json => {
                // I'm really hoping there are no schemas that do this...
                Err(())
            }
        }
    }
}
