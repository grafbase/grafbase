use extension_catalog::{ExtensionCatalog, ExtensionId, KindDiscriminants};
use gateway_config::{AuthenticationProvider, Config};
use runtime::extension::AuthorizerId;

use crate::extension::{ExtensionGuestConfig, SchemaDirective};

use super::{ExtensionConfig, ExtensionPoolId, WasmConfig};

pub(super) fn load_extensions_config(
    extension_catalog: &ExtensionCatalog,
    config: &Config,
    schema: &engine::Schema,
) -> Vec<ExtensionConfig<Option<toml::Value>>> {
    let mut wasm_extensions = Vec::with_capacity(extension_catalog.len());

    for (id, extension) in extension_catalog.iter().enumerate() {
        let manifest = &extension.manifest;
        let extension_config = config
            .extensions
            .get(manifest.name())
            .expect("we made sure in the create_extension_catalog that this extension is in the config");

        let wasi_config = WasmConfig {
            location: extension.wasm_path.clone(),
            networking: extension_config.networking().unwrap_or(manifest.network_enabled()),
            stdout: extension_config.stdout().unwrap_or(manifest.stdout_enabled()),
            stderr: extension_config.stderr().unwrap_or(manifest.stderr_enabled()),
            environment_variables: extension_config
                .environment_variables()
                .unwrap_or(manifest.environment_variables_enabled()),
        };

        let max_pool_size = extension_config.max_pool_size();
        let id = ExtensionId::from(id);

        let r#type = KindDiscriminants::from(&manifest.kind);
        match r#type {
            KindDiscriminants::Resolver => {
                let id = ExtensionPoolId::Resolver(id);

                wasm_extensions.push(ExtensionConfig {
                    id,
                    manifest_id: manifest.id.clone(),
                    max_pool_size,
                    wasm_config: wasi_config,
                    guest_config: ExtensionGuestConfig {
                        r#type,
                        schema_directives: Vec::new(),
                        configuration: extension_config.config().cloned(),
                    },
                    sdk_version: manifest.sdk_version.clone(),
                });
            }
            KindDiscriminants::Authentication => {
                let Some(auth_config) = config.authentication.as_ref() else {
                    continue;
                };

                for (auth_id, provider) in auth_config.providers.iter().enumerate() {
                    let AuthenticationProvider::Extension(extension_provider) = provider else {
                        continue;
                    };

                    if extension_provider.extension != manifest.name() {
                        continue;
                    }

                    let id = ExtensionPoolId::Authorizer(id, AuthorizerId::from(auth_id));

                    wasm_extensions.push(ExtensionConfig {
                        id,
                        manifest_id: manifest.id.clone(),
                        max_pool_size,
                        wasm_config: wasi_config.clone(),
                        guest_config: ExtensionGuestConfig {
                            r#type,
                            schema_directives: Vec::new(),
                            configuration: extension_provider.config.clone(),
                        },
                        sdk_version: manifest.sdk_version.clone(),
                    });
                }
            }
            KindDiscriminants::Authorization => {
                let id = ExtensionPoolId::Authorization(id);

                wasm_extensions.push(ExtensionConfig {
                    id,
                    manifest_id: manifest.id.clone(),
                    max_pool_size,
                    wasm_config: wasi_config,
                    guest_config: ExtensionGuestConfig {
                        r#type,
                        schema_directives: Vec::new(),
                        configuration: extension_config.config().cloned(),
                    },
                    sdk_version: manifest.sdk_version.clone(),
                });
            }
        }
    }

    for subgraph in schema.subgraphs() {
        let directives = subgraph.extension_schema_directives();

        for schema_directive in directives {
            let config = &mut wasm_extensions[usize::from(schema_directive.extension_id)];

            config.guest_config.schema_directives.push(SchemaDirective::new(
                schema_directive.name(),
                subgraph.name(),
                schema_directive.static_arguments(),
            ));
        }
    }

    wasm_extensions
}
