#![allow(dead_code)]

use super::async_client::AsyncClient;
use super::kill_with_children::kill_with_children;
use super::{cargo_bin::cargo_bin, client::Client};
use backend::project::ConfigType;
use cfg_if::cfg_if;
use common::consts::{GRAFBASE_DIRECTORY_NAME, GRAFBASE_SCHEMA_FILE_NAME};
use duct::{cmd, Handle};
use std::env::VarError;
use std::path::Path;
use std::process::Output;
use std::sync::Arc;
use std::{env, fs, io::Write, path::PathBuf};
use std::{io, mem};
use tempfile::{tempdir, TempDir};

pub struct Environment {
    pub endpoint: String,
    pub playground_endpoint: String,
    pub directory: PathBuf,
    pub port: u16,
    temp_dir: Option<Arc<TempDir>>,
    schema_path: PathBuf,
    commands: Vec<Handle>,
    home: Option<PathBuf>,
    ts_config_dependencies_prepared: bool,
    #[cfg(feature = "dynamodb")]
    dynamodb_env: dynamodb::DynamoDbEnvironment,
}

const DOT_ENV_FILE: &str = ".env";

fn get_free_port() -> u16 {
    const INITIAL_PORT: u16 = 4000;

    let test_state_directory_path = std::env::temp_dir().join("grafbase/cli-tests");
    std::fs::create_dir_all(&test_state_directory_path).unwrap();
    let lock_file_path = test_state_directory_path.join("port-number.lock");
    let port_number_file_path = test_state_directory_path.join("port-number.txt");
    let mut lock_file = fslock::LockFile::open(&lock_file_path).unwrap();
    lock_file.lock().unwrap();
    let port_number = if port_number_file_path.exists() {
        std::fs::read_to_string(&port_number_file_path)
            .unwrap()
            .trim()
            .parse::<u16>()
            .unwrap()
            + 1
    } else {
        INITIAL_PORT
    };
    std::fs::write(&port_number_file_path, port_number.to_string()).unwrap();
    lock_file.unlock().unwrap();
    port_number
}

impl Environment {
    #[allow(clippy::needless_return, clippy::unused_async)]
    pub fn init() -> Self {
        let port = get_free_port();
        cfg_if!(
            if #[cfg(feature = "dynamodb")] {
                return tokio::runtime::Runtime::new().unwrap().block_on(async move {
                    let dynamodb_env = dynamodb::DynamoDbEnvironment::new(port).await;
                    Self::init_internal(port, dynamodb_env)
                });
            } else {
                return Self::init_internal(port);
            }
        );
    }

    #[allow(clippy::needless_return, clippy::unused_async)]
    pub async fn init_async() -> Self {
        let port = get_free_port();
        cfg_if!(
            if #[cfg(feature = "dynamodb")] {
                let dynamodb_env = dynamodb::DynamoDbEnvironment::new(port).await;
                return Self::init_internal(port, dynamodb_env);
            } else {
                return Self::init_internal(port);
            }
        );
    }

    fn init_internal(port: u16, #[cfg(feature = "dynamodb")] dynamodb_env: dynamodb::DynamoDbEnvironment) -> Self {
        let temp_dir = tempdir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        let schema_path = temp_dir
            .path()
            .join(GRAFBASE_DIRECTORY_NAME)
            .join(GRAFBASE_SCHEMA_FILE_NAME);
        let directory = temp_dir.path().to_owned();
        println!("Using temporary directory {:?}", directory.as_os_str());
        let commands = vec![];
        let endpoint = format!("http://127.0.0.1:{port}/graphql");
        let playground_endpoint = format!("http://127.0.0.1:{port}");
        let keep_temp_dir = std::env::var("KEEP_TEMP_DIR")
            .and_then(|val| val.parse().map_err(|_| VarError::NotPresent))
            .unwrap_or_default();
        let temp_dir = if keep_temp_dir {
            mem::forget(temp_dir);
            None
        } else {
            Some(Arc::new(temp_dir))
        };
        Self {
            endpoint,
            playground_endpoint,
            directory,
            port,
            temp_dir,
            schema_path,
            commands,
            home: None,
            ts_config_dependencies_prepared: false,
            #[cfg(feature = "dynamodb")]
            dynamodb_env,
        }
    }

    /// Same environment but different port.
    pub fn from(other: &Environment) -> Self {
        let port = get_free_port();
        let temp_dir = other.temp_dir.clone();
        let endpoint = format!("http://127.0.0.1:{port}/graphql");
        let playground_endpoint = format!("http://127.0.0.1:{port}");
        let commands = vec![];

        Self {
            directory: other.directory.clone(),
            commands,
            endpoint,
            playground_endpoint,
            schema_path: other.schema_path.clone(),
            temp_dir,
            port,
            home: other.home.clone(),
            ts_config_dependencies_prepared: other.ts_config_dependencies_prepared,
            #[cfg(feature = "dynamodb")]
            dynamodb_env: dynamodb::DynamoDbEnvironment {
                dynamodb_client: None, // Only one dynamodb client is needed for the cleanup.
                table_name: other.dynamodb_env.table_name.clone(),
            },
        }
    }

    pub fn create_client_with_options(&self, options: super::client::ClientOptions) -> Client {
        Client::new(self.endpoint.clone(), self.playground_endpoint.clone(), options)
    }

    pub fn create_client(&self) -> Client {
        Client::new(
            self.endpoint.clone(),
            self.playground_endpoint.clone(),
            super::client::ClientOptions::default(),
        )
    }

    pub fn create_async_client(&self) -> AsyncClient {
        AsyncClient::new(self.endpoint.clone(), self.playground_endpoint.clone())
    }

    // TODO: change this to set_schema
    pub fn write_schema(&self, schema: impl AsRef<str>) {
        self.write_file("schema.graphql", schema);
    }

    #[cfg(not(target_os = "windows"))]
    pub fn prepare_ts_config_dependencies(&mut self) {
        if self.ts_config_dependencies_prepared {
            return;
        }
        fs::write("package.json", include_str!("../assets/sdk-package.json")).unwrap();
        cmd!("npm", "install").run().unwrap();
        self.ts_config_dependencies_prepared = true;
    }

    #[cfg(not(target_os = "windows"))]
    pub fn set_typescript_config(&mut self, config: impl AsRef<str>) {
        self.prepare_ts_config_dependencies();
        self.write_file("grafbase.config.ts", config);
    }

    #[track_caller]
    pub fn write_resolver(&self, path: impl AsRef<Path>, contents: impl AsRef<str>) {
        self.write_file(Path::new("resolvers").join(path.as_ref()), contents);
    }

    #[track_caller]
    pub fn write_authorizer(&self, path: impl AsRef<Path>, contents: impl AsRef<str>) {
        self.write_file(Path::new("auth").join(path.as_ref()), contents);
    }

    #[track_caller]
    pub fn write_file(&self, path: impl AsRef<Path>, contents: impl AsRef<str>) {
        let target_path = self.schema_path.parent().unwrap().join(path.as_ref());
        fs::create_dir_all(target_path.parent().unwrap()).unwrap();
        fs::write(target_path, contents.as_ref()).unwrap();
    }

    #[track_caller]
    pub fn write_json_file_to_project(&self, path: impl AsRef<Path>, contents: &serde_json::Value) {
        let contents = serde_json::to_string_pretty(contents).unwrap();
        let target_path = self.directory.join(path.as_ref());
        fs::create_dir_all(target_path.parent().unwrap()).unwrap();
        fs::write(target_path, contents).unwrap();
    }

    #[track_caller]
    pub fn load_file_from_project(&self, path: impl AsRef<Path>) -> String {
        fs::read_to_string(self.directory.join(path.as_ref())).unwrap()
    }

    #[track_caller]
    pub fn grafbase_introspect(&self, url: &str, headers: &[&str]) -> Output {
        let mut args = vec!["subgraph", "introspect", url];

        for header in headers {
            args.push("--header");
            args.push(*header);
        }

        duct::cmd(cargo_bin("grafbase"), args)
            .dir(&self.directory)
            .stdout_capture()
            .stderr_capture()
            .unchecked()
            .run()
            .unwrap()
    }

    #[track_caller]
    pub fn grafbase_init(&self, config_format: ConfigType) {
        cmd!(
            cargo_bin("grafbase"),
            "--trace",
            "2",
            "init",
            "-c",
            config_format.as_ref()
        )
        .dir(&self.directory)
        .run()
        .unwrap();
    }

    #[track_caller]
    pub fn grafbase_init_output(&self, config_format: ConfigType) -> Output {
        cmd!(
            cargo_bin("grafbase"),
            "--trace",
            "2",
            "init",
            "-c",
            config_format.as_ref()
        )
        .dir(&self.directory)
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap()
    }

    pub fn grafbase_init_template_output(&self, name: Option<&str>, template: &str) -> Output {
        if let Some(name) = name {
            cmd!(cargo_bin("grafbase"), "init", name, "--template", template)
        } else {
            cmd!(cargo_bin("grafbase"), "init", "--template", template)
        }
        .dir(&self.directory)
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap()
    }

    pub fn grafbase_link_non_interactive(&self, project: &str) -> Output {
        cmd!(cargo_bin("grafbase"), "link", "--project", project)
            .dir(&self.directory)
            .stdout_capture()
            .stderr_capture()
            .unchecked()
            .run()
            .unwrap()
    }

    pub fn grafbase_init_template(&self, name: Option<&str>, template: &str) {
        if let Some(name) = name {
            cmd!(cargo_bin("grafbase"), "init", name, "--template", template)
        } else {
            cmd!(cargo_bin("grafbase"), "init", "--template", template)
        }
        .dir(&self.directory)
        .run()
        .unwrap();
    }

    pub fn remove_grafbase_dir(&self, name: Option<&str>) {
        let directory = name.map_or_else(|| self.directory.join("grafbase"), |name| self.directory.join(name));
        fs::remove_dir_all(directory).unwrap();
    }

    pub fn with_home(mut self, path: PathBuf) -> Self {
        fs::create_dir_all(self.directory.join(&path)).unwrap();
        self.home = Some(path);
        self
    }

    pub fn grafbase_dev(&mut self) {
        let command = cmd!(
            cargo_bin("grafbase"),
            "--trace",
            "2",
            "dev",
            "--disable-watch",
            "--port",
            self.port.to_string()
        )
        .dir(&self.directory);
        #[cfg(feature = "dynamodb")]
        let command = command.env("DYNAMODB_TABLE_NAME", &self.dynamodb_env.table_name);
        let command = command.start().unwrap();

        self.commands.push(command);
    }

    pub fn grafbase_start(&mut self) {
        let command = cmd!(
            cargo_bin("grafbase"),
            "--trace",
            "2",
            "start",
            "--port",
            self.port.to_string()
        )
        .dir(&self.directory);
        #[cfg(feature = "dynamodb")]
        let command = command.env("DYNAMODB_TABLE_NAME", &self.dynamodb_env.table_name);
        let command = command.start().unwrap();

        self.commands.push(command);
    }

    pub fn grafbase_dev_with_home_flag(&mut self) {
        let command = cmd!(
            cargo_bin("grafbase"),
            "--trace",
            "2",
            "--home",
            self.home
                .clone()
                .expect("self.home must be set first")
                .to_string_lossy()
                .to_string(),
            "dev",
            "--disable-watch",
            "--port",
            self.port.to_string()
        )
        .dir(&self.directory)
        .start()
        .unwrap();

        self.commands.push(command);
    }

    pub fn grafbase_dev_output(&mut self) -> io::Result<Output> {
        let command = cmd!(
            cargo_bin("grafbase"),
            "dev",
            "--disable-watch",
            "--port",
            self.port.to_string()
        )
        .dir(&self.directory);
        #[cfg(feature = "dynamodb")]
        let command = command.env("DYNAMODB_TABLE_NAME", &self.dynamodb_env.table_name);
        command.start()?.into_output()
    }

    pub fn set_variables<K, V>(&mut self, variables: impl IntoIterator<Item = (K, V)>)
    where
        K: std::fmt::Display,
        V: std::fmt::Display,
    {
        let env_file = variables
            .into_iter()
            .map(|(key, value)| format!(r#"{key}="{value}""#))
            .collect::<Vec<_>>()
            .join("\n");

        std::fs::write(
            self.schema_path.parent().expect("must exist").join(DOT_ENV_FILE),
            env_file,
        )
        .unwrap();
    }

    pub fn grafbase_reset(&mut self) {
        cmd!(cargo_bin("grafbase"), "reset").dir(&self.directory).run().unwrap();
    }

    pub fn grafbase_dev_watch(&mut self) {
        let command = cmd!(
            cargo_bin("grafbase"),
            "--trace",
            "2",
            "dev",
            "--port",
            self.port.to_string()
        )
        .dir(&self.directory);
        #[cfg(feature = "dynamodb")]
        let command = command.env("DYNAMODB_TABLE_NAME", &self.dynamodb_env.table_name);
        let command = command.start().unwrap();

        self.commands.push(command);
    }

    pub fn append_to_schema(&self, contents: &'static str) {
        let mut file = fs::OpenOptions::new().append(true).open(&self.schema_path).unwrap();

        file.write_all(format!("\n{contents}").as_bytes()).unwrap();

        file.sync_all().unwrap();

        drop(file);
    }

    pub fn kill_processes(&mut self) {
        self.commands.iter().for_each(|command| {
            kill_with_children(*command.pids().first().unwrap());
        });

        self.commands = vec![];
    }

    pub fn has_database_directory(&mut self) -> bool {
        fs::metadata(self.directory.join(".grafbase/database")).is_ok()
    }
}

#[cfg(feature = "dynamodb")]
mod dynamodb {
    use rusoto_dynamodb::{CreateTableInput, DeleteTableInput, DescribeTableInput, DynamoDb};

    pub struct DynamoDbEnvironment {
        pub dynamodb_client: Option<rusoto_dynamodb::DynamoDbClient>, // If set, will be used for db cleanup on drop.
        pub table_name: String,
    }

    impl DynamoDbEnvironment {
        pub async fn new(port: u16) -> Self {
            let table_name = format!("gateway_test_{port}");
            let dynamodb_client = create_database(&table_name).await;
            Self {
                dynamodb_client: Some(dynamodb_client),
                table_name,
            }
        }
    }

    pub async fn create_database(table_name: &String) -> rusoto_dynamodb::DynamoDbClient {
        use rusoto_utils::{attr_def, gsi, key_schema};

        let aws_access_key_id = std::env::var("AWS_ACCESS_KEY_ID").unwrap();
        let aws_secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY").unwrap();
        let dynamodb_region = std::env::var("DYNAMODB_REGION").unwrap();

        let dynamodb_region = match dynamodb_region.strip_prefix("custom:") {
            Some(suffix) => rusoto_core::Region::Custom {
                name: "local".to_string(),
                endpoint: suffix.to_string(),
            },
            None => <rusoto_core::Region as std::str::FromStr>::from_str(&dynamodb_region).unwrap(),
        };
        let aws_credentials =
            rusoto_core::credential::AwsCredentials::new(aws_access_key_id, aws_secret_access_key, None, None);

        let dynamodb_client = {
            let http_client = rusoto_core::HttpClient::new().expect("failed to create HTTP client");
            let credentials_provider = rusoto_core::credential::StaticProvider::from(aws_credentials);
            rusoto_dynamodb::DynamoDbClient::new_with(http_client, credentials_provider, dynamodb_region)
        };

        println!("Initializing dynamodb table name: {table_name}");

        if dynamodb_client
            .describe_table(DescribeTableInput {
                table_name: table_name.clone(),
            })
            .await
            .is_ok()
        {
            println!("Deleting the table");
            dynamodb_client
                .delete_table(DeleteTableInput {
                    table_name: table_name.clone(),
                })
                .await
                .unwrap();
        }

        dynamodb_client
            .create_table(CreateTableInput {
                table_name: table_name.clone(),
                key_schema: key_schema(""),
                attribute_definitions: attr_def(vec!["__pk", "__sk", "__gsi1pk", "__gsi1sk", "__gsi2pk", "__gsi2sk"]),
                global_secondary_indexes: Some(vec![gsi("gsi1"), gsi("gsi2")]),
                billing_mode: Some("PAY_PER_REQUEST".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();

        dynamodb_client
    }

    pub async fn delete_database_async(dynamodb_client: rusoto_dynamodb::DynamoDbClient, table_name: String) {
        println!("Deleting dynamodb table name: {table_name}");
        dynamodb_client
            .delete_table(DeleteTableInput { table_name })
            .await
            .unwrap();
    }

    mod rusoto_utils {
        use rusoto_dynamodb::{AttributeDefinition, GlobalSecondaryIndex, KeySchemaElement, Projection};

        pub fn attr_def(names: Vec<&str>) -> Vec<AttributeDefinition> {
            names
                .into_iter()
                .map(|name| AttributeDefinition {
                    attribute_name: name.to_string(),
                    attribute_type: "S".to_string(),
                })
                .collect()
        }

        pub fn key_schema(infix: &str) -> Vec<KeySchemaElement> {
            vec![
                KeySchemaElement {
                    attribute_name: format!("__{infix}pk"),
                    key_type: "HASH".to_string(),
                },
                KeySchemaElement {
                    attribute_name: format!("__{infix}sk"),
                    key_type: "RANGE".to_string(),
                },
            ]
        }

        pub fn gsi(name: &str) -> GlobalSecondaryIndex {
            GlobalSecondaryIndex {
                index_name: name.to_string(),
                key_schema: key_schema(name),
                projection: Projection {
                    projection_type: Some("ALL".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        self.kill_processes();
        #[cfg(feature = "dynamodb")]
        if let Some(dynamodb_client) = self.dynamodb_env.dynamodb_client.take() {
            let cleanup_future = dynamodb::delete_database_async(dynamodb_client, self.dynamodb_env.table_name.clone());
            match tokio::runtime::Handle::try_current() {
                Ok(handle) => {
                    tokio::task::block_in_place(|| handle.block_on(cleanup_future));
                }
                Err(_) => {
                    tokio::runtime::Runtime::new().unwrap().block_on(cleanup_future);
                }
            }
        }
    }
}
