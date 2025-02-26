use cynic::{QueryBuilder as _, http::ReqwestExt};

use crate::{
    backend::api::{
        client::create_client,
        graphql::mutations::extension_versions_by_version_requirement::{
            ExtensionVersionRequirement, ExtensionVersionsByVersionRequirement,
            ExtensionVersionsByVersionRequirementVariables,
        },
    },
    common::environment::PlatformData,
};

use super::graphql::mutations::extension_versions_by_version_requirement;

pub(crate) enum ExtensionVersionMatch {
    Match { name: String, version: semver::Version },
    ExtensionDoesNotExist,
    ExtensionVersionDoesNotExist,
}

pub(crate) async fn extension_versions_by_version_requirement(
    requirements: impl Iterator<Item = (String, semver::VersionReq)>,
) -> anyhow::Result<Vec<ExtensionVersionMatch>> {
    let client = create_client().await?;
    let platform = PlatformData::get();

    let requirements = requirements
        .map(|(name, version)| ExtensionVersionRequirement {
            extension_name: name,
            version,
        })
        .collect::<Vec<_>>();

    let operation =
        ExtensionVersionsByVersionRequirement::build(ExtensionVersionsByVersionRequirementVariables { requirements });

    let response = client.post(&platform.api_url).run_graphql(operation).await?;

    let Some(data) = response
        .data
        .and_then(|data| data.extension_versions_by_version_requirement)
    else {
        return Err(anyhow::anyhow!(
            "Failed to fetch extension versions. Errors: {:#?}",
            response.errors
        ));
    };

    data.into_iter()
        .map(|m| match m {
            extension_versions_by_version_requirement::ExtensionVersionMatch::ExtensionVersion(extension_version) => {
                Ok(ExtensionVersionMatch::Match {
                    name: extension_version.extension.name,
                    version: extension_version.version,
                })
            }
            extension_versions_by_version_requirement::ExtensionVersionMatch::ExtensionDoesNotExistError(_) => {
                Ok(ExtensionVersionMatch::ExtensionDoesNotExist)
            }
            extension_versions_by_version_requirement::ExtensionVersionMatch::ExtensionVersionDoesNotExistError(_) => {
                Ok(ExtensionVersionMatch::ExtensionVersionDoesNotExist)
            }
            extension_versions_by_version_requirement::ExtensionVersionMatch::Unknown(err) => {
                Err(anyhow::anyhow!("Unknown response: {err}"))
            }
        })
        .collect()
}
