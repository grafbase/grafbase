use std::fs;

use extension::lockfile;

use crate::{
    backend::api::{self, extension_versions_by_version_requirement::ExtensionVersionMatch},
    cli_input::ExtensionUpdateCommand,
    output::report,
};

#[tokio::main]
pub(super) async fn execute(cmd: ExtensionUpdateCommand) -> anyhow::Result<()> {
    let ExtensionUpdateCommand { name, config } = cmd;

    let config = fs::read_to_string(config).map_err(|err| anyhow::anyhow!("Failed to read config file: {err}"))?;

    let config_toml: gateway_config::Config = toml::from_str(&config)?;

    let names = name.unwrap_or_default();
    let extensions_from_config = config_toml.extensions.unwrap_or_default();

    if extensions_from_config.is_empty() && names.is_empty() {
        println!("No extension to update");
        return Ok(());
    }

    let mut lockfile = if names.is_empty() {
        lockfile::Lockfile::default()
    } else {
        let lockfile::VersionedLockfile::V1(lockfile): lockfile::VersionedLockfile =
            match fs::read_to_string(lockfile::EXTENSION_LOCKFILE_NAME) {
                Ok(contents) => toml::from_str(&contents)?,
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "❌ No lockfile found, please run `grafbase extension update` without --name first"
                    ));
                }
            };

        lockfile
    };

    let mut config_version_requirements: Vec<(String, semver::VersionReq)> = Vec::with_capacity(if names.is_empty() {
        extensions_from_config.len()
    } else {
        names.len()
    });

    if !names.is_empty() {
        for name in &names {
            let Some(version) = extensions_from_config
                .get(name.as_str())
                .map(|ext| ext.version().to_owned())
            else {
                return Err(anyhow::anyhow!(
                    "Extension {name} is not defined in the gateway configuration.",
                ));
            };
            config_version_requirements.push((name.clone(), version));
        }
    } else {
        for (name, config) in extensions_from_config {
            config_version_requirements.push((name, config.version().to_owned()));
        }
    }

    let matches = api::extension_versions_by_version_requirement::extension_versions_by_version_requirement(
        config_version_requirements
            .iter()
            .map(|(name, version)| (name.clone(), version.clone())),
    )
    .await?;

    let new_lockfile: lockfile::Lockfile = if names.is_empty() {
        for (i, m) in matches.into_iter().enumerate() {
            match m {
                ExtensionVersionMatch::Match { name, version } => {
                    lockfile.extensions.push(lockfile::Extension { name, version })
                }
                ExtensionVersionMatch::ExtensionDoesNotExist => {
                    let (name, _req) = &config_version_requirements[i];
                    handle_extension_does_not_exist(name)
                }
                ExtensionVersionMatch::ExtensionVersionDoesNotExist => {
                    let (name, req) = &config_version_requirements[i];

                    handle_extension_version_does_not_exist(name, req)
                }
            }
        }

        lockfile
    } else {
        for (i, m) in matches.into_iter().enumerate() {
            match m {
                ExtensionVersionMatch::Match { name, version } => {
                    match lockfile.extensions.iter_mut().find(|ext| ext.name == name) {
                        Some(entry) => entry.version = version,
                        None => lockfile.extensions.push(lockfile::Extension { name, version }),
                    }
                }
                ExtensionVersionMatch::ExtensionDoesNotExist => {
                    let (name, _req) = &config_version_requirements[i];
                    handle_extension_does_not_exist(name)
                }
                ExtensionVersionMatch::ExtensionVersionDoesNotExist => {
                    let (name, req) = &config_version_requirements[i];

                    handle_extension_version_does_not_exist(name, req)
                }
            }
        }

        lockfile
    };

    let new_versioned_lockfile = lockfile::VersionedLockfile::V1(new_lockfile);

    fs::write(
        lockfile::EXTENSION_LOCKFILE_NAME,
        toml::to_string(&new_versioned_lockfile)?,
    )?;

    Ok(())
}

pub(super) fn handle_extension_does_not_exist(name: &str) {
    report::extension_update_extension_does_not_exist(name);
    std::process::exit(1);
}

pub(super) fn handle_extension_version_does_not_exist(name: &str, version: &semver::VersionReq) {
    report::extension_update_extension_version_does_not_exist(name, version);
    std::process::exit(1);
}
