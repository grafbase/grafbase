use crate::errors::BackendError;
use gateway_config::Config;
use serde_toml_merge::merge;
use std::{collections::HashSet, path::PathBuf};
use tokio::fs;

#[derive(Debug)]
pub struct DevConfiguration {
    pub introspection_forced: bool,
    pub overridden_subgraphs: HashSet<String>,
    pub merged_configuration: Config,
}

pub async fn get_and_merge_configurations(
    gateway_config_path: Option<&PathBuf>,
    graph_overrides_path: Option<&PathBuf>,
) -> Result<DevConfiguration, BackendError> {
    let config_string = match gateway_config_path {
        Some(path) => fs::read_to_string(path)
            .await
            .map_err(BackendError::ReadGatewayConfig)?,
        None => String::new(),
    };

    let (config, config_value): (Config, toml::Value) = {
        let gateway_config_value = config_string
            .parse::<toml::Value>()
            .map_err(BackendError::ParseGatewayConfig)?;

        let config = gateway_config_value
            .clone()
            .try_into()
            .map_err(BackendError::ParseGatewayConfig)?;

        (config, gateway_config_value)
    };

    for (_, subgraph) in config.subgraphs.iter() {
        match (
            &subgraph.introspection_url,
            &subgraph.introspection_headers,
            &subgraph.schema_path,
        ) {
            (Some(_), _, _) => return Err(BackendError::DevOptionsInGatewayConfig("introspection_url")),
            (_, Some(_), _) => return Err(BackendError::DevOptionsInGatewayConfig("introspection_headers")),
            (_, _, Some(_)) => return Err(BackendError::DevOptionsInGatewayConfig("schema_path")),
            _ => {}
        }
    }

    let (graph_overrides, overrides_value): (Option<Config>, Option<toml::Value>) =
        if let Some(ref graph_overrides_path) = graph_overrides_path {
            let graph_overrides_value = fs::read_to_string(graph_overrides_path)
                .await
                .map_err(BackendError::ReadGraphOverrides)?
                .parse::<toml::Value>()
                .map_err(BackendError::ParseGraphOverrides)?;

            let graph_overrides = graph_overrides_value
                .clone()
                .try_into()
                .map_err(BackendError::ParseGraphOverrides)?;

            (Some(graph_overrides), Some(graph_overrides_value))
        } else {
            (None, None)
        };

    let mut merged_configuration = if overrides_value.is_none() {
        config
    } else {
        Some(config_value)
            .zip(overrides_value)
            .map(|(config, overrides)| merge(config, overrides))
            .transpose()
            .map_err(|_| BackendError::MergeConfigurations)?
            .map(|config| config.try_into::<Config>())
            .transpose()
            .map_err(|_| BackendError::MergeConfigurations)?
            .unwrap_or_default()
    };

    let overridden_subgraphs = graph_overrides
        .map(|config| config.subgraphs.into_keys().collect::<HashSet<_>>())
        .unwrap_or_default();

    let introspection_forced = merged_configuration.graph.introspection == Some(false);
    merged_configuration.graph.introspection = Some(true);

    Ok(DevConfiguration {
        introspection_forced,
        overridden_subgraphs,
        merged_configuration,
    })
}
