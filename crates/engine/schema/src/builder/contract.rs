use url::Url;

use crate::{DirectiveSiteId, Schema, UrlId, builder::finalize_inaccessible};

pub struct MutableSchema(Schema);

impl Schema {
    pub fn into_mutable(self) -> MutableSchema {
        MutableSchema(self)
    }
}

impl MutableSchema {
    pub fn mark_all_as_inaccessible(&mut self) {
        self.0.graph.inaccessible_object_definitions.set_all(true);
        self.0.graph.inaccessible_interface_definitions.set_all(true);
        self.0.graph.inaccessible_field_definitions.set_all(true);
        self.0.graph.inaccessible_enum_definitions.set_all(true);
        self.0.graph.inaccessible_enum_values.set_all(true);
        self.0.graph.inaccessible_input_object_definitions.set_all(true);
        self.0.graph.inaccessible_input_value_definitions.set_all(true);
        self.0.graph.inaccessible_scalar_definitions.set_all(true);
        self.0.graph.inaccessible_union_definitions.set_all(true);
    }

    pub fn mark_as_accessible(&mut self, site_id: DirectiveSiteId, accessible: bool) {
        match site_id {
            DirectiveSiteId::Enum(id) => {
                self.0.graph.inaccessible_enum_definitions.set(id, !accessible);
            }
            DirectiveSiteId::EnumValue(id) => {
                self.0.graph.inaccessible_enum_values.set(id, !accessible);
            }
            DirectiveSiteId::Field(id) => {
                self.0.graph.inaccessible_field_definitions.set(id, !accessible);
            }
            DirectiveSiteId::InputObject(id) => {
                self.0.graph.inaccessible_input_object_definitions.set(id, !accessible);
            }
            DirectiveSiteId::InputValue(id) => {
                self.0.graph.inaccessible_input_value_definitions.set(id, !accessible);
            }
            DirectiveSiteId::Interface(id) => {
                self.0.graph.inaccessible_interface_definitions.set(id, !accessible);
            }
            DirectiveSiteId::Object(id) => {
                self.0.graph.inaccessible_object_definitions.set(id, !accessible);
            }
            DirectiveSiteId::Scalar(id) => {
                self.0.graph.inaccessible_scalar_definitions.set(id, !accessible);
            }
            DirectiveSiteId::Union(id) => {
                self.0.graph.inaccessible_union_definitions.set(id, !accessible);
            }
        }
    }

    pub fn update_graphql_endpoint(&mut self, name: &str, url: Url) {
        let Some(id) = self
            .0
            .subgraphs()
            .filter_map(|sg| sg.as_graphql_endpoint())
            .find(|gql| gql.subgraph_name() == name)
            .filter(|gql| gql.url() == &url)
            .map(|gql| gql.id)
        else {
            return;
        };
        let url_id = self.get_or_insert_url(url);
        self.0[id].url_id = url_id;
    }

    fn get_or_insert_url(&mut self, url: Url) -> UrlId {
        if let Some(pos) = self.0.urls.iter().position(|candidate| candidate == &url) {
            return pos.into();
        }
        self.0.urls.push(url);
        UrlId::from(self.0.urls.len() - 1)
    }

    pub fn finalize(mut self) -> Schema {
        // reset
        self.0.graph.union_has_inaccessible_member.set_all(false);
        self.0.graph.interface_has_inaccessible_implementor.set_all(false);
        finalize_inaccessible(&mut self.0.graph);
        self.0
    }
}
