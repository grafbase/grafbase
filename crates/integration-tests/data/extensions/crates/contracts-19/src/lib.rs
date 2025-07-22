use grafbase_sdk::{
    ContractsExtension,
    types::{Configuration, Contract, ContractDirective, Error, GraphqlSubgraph},
};

#[derive(ContractsExtension)]
struct Contracts {
    config: Config,
}

#[derive(Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Config {
    // Apollo defaults to false
    hide_unreacheable_types: bool,
}

impl ContractsExtension for Contracts {
    fn new(config: Configuration) -> Result<Self, Error> {
        Ok(Self {
            config: config.deserialize()?,
        })
    }

    fn construct(
        &mut self,
        key: String,
        directives: Vec<ContractDirective<'_>>,
        _subgraphs: Vec<GraphqlSubgraph>,
    ) -> Result<Contract, Error> {
        let ContractKey {
            included_tags,
            excluded_tags,
        } = serde_json::from_str(&key).map_err(|err| format!("Could not read contract key: {err}"))?;
        let mut contract = Contract::new(&directives);

        // Apollo doc (https://www.apollographql.com/docs/graphos/platform/schema-management/delivery/contracts/create#3-create-a-contract):
        //     - If the Included Tags list is empty, the contract schema includes each type and object/interface field
        //       unless it's tagged with an excluded tag.
        //     - If the Included Tags list is non-empty, the contract schema excludes each union type and object/interface
        //       field unless it's tagged with an included tag.
        //       - Each object and interface type is included as long as at least one of its fields is included
        //         (unless the type is explicitly excluded)
        //       - The contract schema excludes a type or field if it's tagged with both an included tag and an excluded tag.
        //     - If you enable the option to hide unreachable types, the contract schema excludes each unreachable object,
        //       interface, union, input, enum, and scalar unless it's tagged with an included tag.
        contract
            .hide_unreacheable_types(self.config.hide_unreacheable_types)
            .accessible_by_default(!included_tags.is_empty());

        for directive in directives {
            let Tag { name } = directive.arguments()?;
            if excluded_tags.iter().any(|tag| tag == name) {
                contract.override_accessible(directive, false);
            } else if included_tags.iter().any(|tag| tag == name) {
                contract.accessible(directive, true);
            }
        }

        Ok(contract)
    }
}

#[derive(Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct ContractKey {
    included_tags: Vec<String>,
    excluded_tags: Vec<String>,
}

#[derive(serde::Deserialize)]
struct Tag<'a> {
    name: &'a str,
}
