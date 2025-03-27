mod backwards_compatibility;
mod default;
mod multiple;
pub mod static_token;
use std::sync::Arc;

use extension_catalog::Id;
use integration_tests::federation::{TestExtension, TestExtensionBuilder, TestManifest};

pub struct AuthenticationExt<T> {
    instance: Arc<dyn TestExtension>,
    name: &'static str,
    sdl: Option<&'static str>,
    phantom: std::marker::PhantomData<T>,
}

impl<T: TestExtension> AuthenticationExt<T> {
    pub fn new(instance: T) -> Self {
        Self {
            instance: Arc::new(instance),
            name: "authentication",
            sdl: None,
            phantom: std::marker::PhantomData,
        }
    }

    #[allow(unused)]
    #[must_use]
    pub fn with_sdl(mut self, sdl: &'static str) -> Self {
        self.sdl = Some(sdl);
        self
    }

    #[allow(unused)]
    #[must_use]
    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }
}

impl<T: TestExtension> TestExtensionBuilder for AuthenticationExt<T> {
    fn manifest(&self) -> TestManifest {
        TestManifest {
            id: Id {
                name: self.name.to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::Authentication(Default::default()),
            sdl: None,
        }
    }

    fn build(&self, _schema_directives: Vec<(&str, serde_json::Value)>) -> std::sync::Arc<dyn TestExtension> {
        self.instance.clone()
    }
}
