//! Module that deals with namespacing our output within the destination
//! GraphQL schema

use std::collections::BTreeMap;

use dynaql::{
    indexmap::IndexMap,
    registry::{MetaField, MetaType, Registry},
};
use inflector::Inflector;

use crate::ApiMetadata;

use super::{meta_field, object};

pub trait RegistryExt {
    fn query_fields(&mut self, api_metadata: &ApiMetadata) -> &mut IndexMap<String, MetaField>;
    fn mutation_fields(&mut self, api_metadata: &ApiMetadata) -> &mut IndexMap<String, MetaField>;
}

impl RegistryExt for Registry {
    fn query_fields(&mut self, api_metadata: &ApiMetadata) -> &mut IndexMap<String, MetaField> {
        let object_name = format!("{}Queries", api_metadata.name.to_pascal_case());

        insert_field(
            api_metadata.name.to_camel_case(),
            object_name.clone(),
            self.query_root_mut().fields_mut().expect("QueryRoot to be an Object"),
        );

        insert_empty_object(object_name, &mut self.types)
    }

    fn mutation_fields(&mut self, api_metadata: &ApiMetadata) -> &mut IndexMap<String, MetaField> {
        if self.mutation_type.is_none() {
            let name = "Mutation".to_string();
            self.mutation_type = Some(name.clone());
            insert_empty_object(name, &mut self.types);
        }

        let object_name = format!("{}Mutations", api_metadata.name.to_pascal_case());

        insert_field(
            api_metadata.name.to_camel_case(),
            object_name.clone(),
            self.mutation_root_mut()
                .fields_mut()
                .expect("MutationRoot to be an Object"),
        );

        insert_empty_object(object_name, &mut self.types)
    }
}

fn insert_field(name: String, ty: String, fields: &mut IndexMap<String, MetaField>) {
    let field = meta_field(name, ty);
    fields.insert(field.name.clone(), field);
}

fn insert_empty_object(name: String, types: &mut BTreeMap<String, MetaType>) -> &mut IndexMap<String, MetaField> {
    types.insert(name.clone(), object(name.clone(), vec![]));
    types.get_mut(&name).unwrap().fields_mut().unwrap()
}
