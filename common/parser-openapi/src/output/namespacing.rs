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
    fn query_fields_mut(&mut self, api_metadata: &ApiMetadata) -> &mut IndexMap<String, MetaField>;
    fn mutation_fields_mut(&mut self, api_metadata: &ApiMetadata) -> &mut IndexMap<String, MetaField>;
}

impl RegistryExt for Registry {
    fn query_fields_mut(&mut self, api_metadata: &ApiMetadata) -> &mut IndexMap<String, MetaField> {
        let object_name = format!("{}Query", api_metadata.namespace.to_pascal_case());

        insert_field(
            self.query_root_mut().fields_mut().expect("QueryRoot to be an Object"),
            api_metadata.namespace.to_camel_case(),
            format!("{object_name}!"),
        );

        insert_empty_object(&mut self.types, object_name)
    }

    fn mutation_fields_mut(&mut self, api_metadata: &ApiMetadata) -> &mut IndexMap<String, MetaField> {
        if self.mutation_type.is_none() {
            let name = "Mutation".to_string();
            self.mutation_type = Some(name.clone());
            insert_empty_object(&mut self.types, name);
        }

        let object_name = format!("{}Mutation", api_metadata.namespace.to_pascal_case());

        insert_field(
            self.mutation_root_mut()
                .fields_mut()
                .expect("MutationRoot to be an Object"),
            api_metadata.namespace.to_camel_case(),
            format!("{object_name}!"),
        );

        insert_empty_object(&mut self.types, object_name)
    }
}

fn insert_field(fields: &mut IndexMap<String, MetaField>, name: String, ty: String) {
    let field = meta_field(name, ty);
    fields.insert(field.name.clone(), field);
}

fn insert_empty_object(types: &mut BTreeMap<String, MetaType>, name: String) -> &mut IndexMap<String, MetaField> {
    types.insert(name.clone(), object(name.clone(), vec![]));
    types.get_mut(&name).unwrap().fields_mut().unwrap()
}
