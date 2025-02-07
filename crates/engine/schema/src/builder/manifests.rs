use std::str::FromStr as _;

use super::{BuildError, Context};

impl Context<'_> {
    pub(super) async fn load_manifests(&mut self) -> Result<(), BuildError> {
        for (ix, extension) in self.federated_graph.extensions.iter().enumerate() {
            let url_str = &self.federated_graph[extension.url];
            let url =
                url::Url::from_str(&self.federated_graph[extension.url]).map_err(|err| BuildError::InvalidUrl {
                    url: url_str.to_string(),
                    err: err.to_string(),
                })?;
            let manifest =
                extension_catalog::load_manifest(url)
                    .await
                    .map_err(|err| BuildError::CouldNotLoadExtension {
                        url: url_str.to_string(),
                        err: err.to_string(),
                    })?;
            let Some(id) = self.extension_catalog.find_compatible_extension(&manifest.id) else {
                return Err(BuildError::UnsupportedExtension {
                    id: Box::new(manifest.id.clone()),
                });
            };
            self.extension_manifests.push(manifest);
            self.extension_mapping.insert(ix.into(), id);
        }

        Ok(())
    }
}
