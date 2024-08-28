use std::{
    borrow::Cow,
    fs,
    future::Future,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::Arc,
};

use duct::cmd;
use tempfile::tempdir;

use crate::{cargo::cargo_bin, client::Client, listen_address, runtime, CommandHandles};

pub struct GatewayBuilder<'a> {
    pub toml_config: ConfigContent<'a>,
    pub schema: &'a str,
    pub log_level: Option<String>,
    pub client_url_path: Option<&'a str>,
    pub client_headers: Option<&'static [(&'static str, &'static str)]>,
}

pub struct ConfigContent<'a>(pub Option<Cow<'a, str>>);

impl<'a> From<&'a str> for ConfigContent<'a> {
    fn from(s: &'a str) -> Self {
        ConfigContent(Some(Cow::Borrowed(s)))
    }
}

impl<'a> From<&'a String> for ConfigContent<'a> {
    fn from(s: &'a String) -> Self {
        ConfigContent(Some(Cow::Borrowed(s)))
    }
}

impl From<String> for ConfigContent<'static> {
    fn from(s: String) -> Self {
        ConfigContent(Some(Cow::Owned(s)))
    }
}

impl<'a> GatewayBuilder<'a> {
    pub fn new(schema: &'a str) -> Self {
        Self {
            toml_config: ConfigContent(None),
            schema,
            log_level: None,
            client_url_path: None,
            client_headers: None,
        }
    }

    pub fn with_log_level(mut self, level: &str) -> Self {
        self.log_level = Some(level.into());
        self
    }

    pub fn run<F>(self, test: impl FnOnce(Arc<Client>) -> F)
    where
        F: Future<Output = ()>,
    {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("grafbase.toml");

        let schema_path = temp_dir.path().join("schema.graphql");
        fs::write(&schema_path, self.schema).unwrap();

        let addr = listen_address();
        let mut args = vec![
            "--listen-address".to_string(),
            addr.to_string(),
            "--schema".to_string(),
            schema_path.to_str().unwrap().to_string(),
        ];

        if let Some(config) = self.toml_config.0 {
            fs::write(&config_path, config.as_ref()).unwrap();
            args.push("--config".to_string());
            args.push(config_path.to_str().unwrap().to_string());
        }

        if let Some(level) = self.log_level {
            args.push("--log".to_string());
            args.push(level);
        }

        let command = cmd(cargo_bin("grafbase-gateway"), &args).stdout_null().stderr_null();

        let endpoint = match self.client_url_path {
            Some(path) => format!("http://{addr}/{path}"),
            None => format!("http://{addr}/graphql"),
        };

        let mut commands = CommandHandles::new();
        commands.push(command.start().unwrap());

        let mut client = Client::new(endpoint, commands);

        if let Some(headers) = self.client_headers {
            for header in headers {
                client = client.with_header(header.0, header.1);
            }
        }

        let client = Arc::new(client);

        let res = catch_unwind(AssertUnwindSafe(|| {
            runtime().block_on(async {
                client.poll_endpoint(30, 300).await;
                test(client.clone()).await
            })
        }));

        client.kill_handles();

        if let Err(err) = res {
            std::panic::resume_unwind(err);
        }
    }
}
