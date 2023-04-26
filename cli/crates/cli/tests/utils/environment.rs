#![allow(dead_code)]

use super::async_client::AsyncClient;
use super::kill_with_children::kill_with_children;
use super::{cargo_bin::cargo_bin, client::Client};
use common::consts::{DOT_GRAFBASE_DIRECTORY, GRAFBASE_DIRECTORY_NAME, GRAFBASE_SCHEMA_FILE_NAME};
use duct::{cmd, Handle};
use std::io;
use std::path::Path;
use std::process::Output;
use std::sync::Arc;
use std::{env, fs, io::Write, path::PathBuf};
use tempfile::{tempdir, TempDir};

pub struct Environment {
    pub endpoint: String,
    pub playground_endpoint: String,
    pub directory: PathBuf,
    pub port: u16,
    temp_dir: Arc<TempDir>,
    schema_path: PathBuf,
    commands: Vec<Handle>,
    // cfg dynamodb
    dynamodb_client: Option<rusoto_dynamodb::DynamoDbClient>, // If set, will be used for db cleanup on drop.
    wrangler: toml::Value, // When grafbase init is called, replace wrangler.toml with the correct db configuration.
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
    pub fn init() -> Self {
        tokio::runtime::Runtime::new().unwrap().block_on(async move {
            let port = get_free_port();
            let (dynamodb_client, wrangler) = Self::create_database(port).await;
            Self::init_internal(Some(dynamodb_client), wrangler, port).await
        })
    }

    pub async fn init_async() -> (Self, impl std::future::Future<Output = ()>) {
        let port = get_free_port();
        let (dynamodb_client, wrangler) = Self::create_database(port).await;
        let ret = Self::init_internal(None, wrangler, port).await;
        let cleanup_fut = Self::delete_database_async(dynamodb_client, port);
        (ret, cleanup_fut)
    }

    async fn init_internal(
        dynamodb_client: Option<rusoto_dynamodb::DynamoDbClient>,
        wrangler: toml::Value,
        port: u16,
    ) -> Self {
        let temp_dir = Arc::new(tempdir().unwrap());
        env::set_current_dir(temp_dir.path()).unwrap();

        let schema_path = temp_dir
            .path()
            .join(GRAFBASE_DIRECTORY_NAME)
            .join(GRAFBASE_SCHEMA_FILE_NAME);
        let directory = temp_dir.path().to_owned();
        println!("Using temporary directory {:?}", directory.as_os_str());
        Self {
            directory,
            commands: vec![],
            endpoint: format!("http://127.0.0.1:{port}/graphql"),
            playground_endpoint: format!("http://127.0.0.1:{port}"),
            schema_path,
            temp_dir,
            port,
            dynamodb_client,
            wrangler,
        }
    }

    /// Same environment but different port.
    pub fn from(other: &Environment) -> Self {
        let port = get_free_port();

        let temp_dir = other.temp_dir.clone();

        Self {
            directory: other.directory.clone(),
            commands: vec![],
            endpoint: format!("http://127.0.0.1:{port}/graphql"),
            playground_endpoint: format!("http://127.0.0.1:{port}"),
            schema_path: other.schema_path.clone(),
            temp_dir,
            port,
            dynamodb_client: None, // Only one dynamodb client is needed for the cleanup.
            wrangler: other.wrangler.clone(),
        }
    }

    pub fn create_client(&self) -> Client {
        Client::new(self.endpoint.clone(), self.playground_endpoint.clone())
    }

    pub fn create_async_client(&self) -> AsyncClient {
        AsyncClient::new(self.endpoint.clone(), self.playground_endpoint.clone())
    }

    pub fn write_schema(&self, schema: impl AsRef<str>) {
        self.write_file("schema.graphql", schema);
    }

    pub fn write_resolver(&self, path: impl AsRef<Path>, contents: impl AsRef<str>) {
        self.write_file(Path::new("resolvers").join(path.as_ref()), contents);
    }

    pub fn write_file(&self, path: impl AsRef<Path>, contents: impl AsRef<str>) {
        let target_path = self.schema_path.parent().unwrap().join(path.as_ref());
        fs::create_dir_all(target_path.parent().unwrap()).unwrap();
        fs::write(target_path, contents.as_ref()).unwrap();
    }

    pub fn grafbase_init(&self) {
        cmd!(cargo_bin("grafbase"), "--nohome", "init")
            .dir(&self.directory)
            .run()
            .unwrap();
        self.update_wrangler_toml(None);
    }

    pub fn grafbase_init_output(&self) -> Output {
        let output = cmd!(cargo_bin("grafbase"), "--nohome", "init")
            .dir(&self.directory)
            .stderr_capture()
            .unchecked()
            .run()
            .unwrap();
        self.update_wrangler_toml(None);
        output
    }

    pub fn grafbase_init_template_output(&self, name: Option<&str>, template: &str) -> Output {
        let output = if let Some(name) = name {
            cmd!(cargo_bin("grafbase"), "--nohome", "init", name, "--template", template)
        } else {
            cmd!(cargo_bin("grafbase"), "--nohome", "init", "--template", template)
        }
        .dir(&self.directory)
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap();
        if output.status.success() {
            self.update_wrangler_toml(name);
        }
        output
    }

    pub fn grafbase_init_template(&self, name: Option<&str>, template: &str) {
        if let Some(name) = name {
            cmd!(cargo_bin("grafbase"), "--nohome", "init", name, "--template", template)
        } else {
            cmd!(cargo_bin("grafbase"), "--nohome", "init", "--template", template)
        }
        .dir(&self.directory)
        .run()
        .unwrap();
        self.update_wrangler_toml(name);
    }

    fn dynamodb_table_name(port: u16) -> String {
        format!("gateway_test_{port}")
    }

    fn update_wrangler_toml(&self, project_name: Option<&str>) {
        let dot_grafbase_path = if let Some(project_name) = project_name {
            self.directory.clone().join(project_name)
        } else {
            self.directory.clone() // TODO join "."
        }
        .join(GRAFBASE_DIRECTORY_NAME)
        .join(DOT_GRAFBASE_DIRECTORY);
        assert!(dot_grafbase_path.exists(), "{dot_grafbase_path:?} must exist");
        let wrangler_toml_path = dot_grafbase_path.join("wrangler.toml");
        assert!(wrangler_toml_path.exists(), "{wrangler_toml_path:?} must exist");
        let content = toml::to_string(&self.wrangler).expect("wrangler.toml must be serializable");
        std::fs::write(wrangler_toml_path, content).expect("saving wrangler.toml must succeed");
    }

    async fn create_database(port: u16) -> (rusoto_dynamodb::DynamoDbClient, toml::Value) {
        // Read dynamo configuration from assets' wrangler.toml
        let wrangler_toml = server::types::Assets::get("wrangler.toml")
            .expect("wrangler.toml must exist")
            .data;
        let wrangler_toml = String::from_utf8(wrangler_toml.into_owned()).expect("wrangler.toml must be in UTF-8");
        let mut toml: toml::Table = toml::from_str(&wrangler_toml).expect("toml must be parseable");

        let vars = toml
            .get_mut("vars")
            .expect("vars must exist")
            .as_table_mut()
            .expect("vars must be a table");
        fn get<'a>(vars: &'a toml::map::Map<String, toml::Value>, key: &str) -> &'a str {
            vars.get(key)
                .unwrap_or_else(|| panic!("{key} must exist"))
                .as_str()
                .unwrap_or_else(|| panic!("{key} must be a string"))
        }

        let table_name = Self::dynamodb_table_name(port);
        vars.insert(
            "DYNAMODB_TABLE_NAME".to_string(),
            toml::Value::String(table_name.clone()),
        );
        let aws_access_key_id = get(vars, "AWS_ACCESS_KEY_ID");
        let aws_secret_access_key = get(vars, "AWS_SECRET_ACCESS_KEY");
        let dynamodb_region = get(vars, "DYNAMODB_REPLICATION_REGIONS");

        let dynamodb_region = match dynamodb_region.strip_prefix("custom:") {
            Some(suffix) => rusoto_core::Region::Custom {
                name: "local".to_string(),
                endpoint: suffix.to_string(),
            },
            None => <rusoto_core::Region as std::str::FromStr>::from_str(dynamodb_region).unwrap(),
        };
        let aws_credentials =
            rusoto_core::credential::AwsCredentials::new(aws_access_key_id, aws_secret_access_key, None, None);

        let dynamodb_client = {
            let http_client = rusoto_core::HttpClient::new().expect("failed to create HTTP client");
            let credentials_provider = rusoto_core::credential::StaticProvider::from(aws_credentials);
            rusoto_dynamodb::DynamoDbClient::new_with(http_client, credentials_provider, dynamodb_region)
        };

        println!("Initializing dynamodb table name: {table_name}");
        // let dynamodb_client =
        //tokio::runtime::Runtime::new().unwrap().block_on(async move {
        use rusoto_dynamodb::{
            AttributeDefinition, CreateTableInput, DeleteTableInput, DescribeTableInput, DynamoDb,
            GlobalSecondaryIndex, KeySchemaElement, Projection,
        };
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
        fn attr_def(names: Vec<&str>) -> Vec<AttributeDefinition> {
            names
                .into_iter()
                .map(|name| AttributeDefinition {
                    attribute_name: name.to_string(),
                    attribute_type: "S".to_string(),
                })
                .collect()
        }
        fn key_schema(infix: &str) -> Vec<KeySchemaElement> {
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
        fn gsi(name: &str) -> GlobalSecondaryIndex {
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
        //     dynamodb_client
        // });
        (dynamodb_client, toml::Value::Table(toml))
    }

    async fn delete_database_async(dynamodb_client: rusoto_dynamodb::DynamoDbClient, port: u16) {
        use rusoto_dynamodb::{DeleteTableInput, DynamoDb};
        let table_name = Self::dynamodb_table_name(port);
        println!("Deleting dynamodb table name: {table_name}");
        dynamodb_client
            .delete_table(DeleteTableInput {
                table_name: table_name.clone(),
            })
            .await
            .unwrap();
    }

    pub fn remove_grafbase_dir(&self, name: Option<&str>) {
        let directory = name.map_or_else(|| self.directory.join("grafbase"), |name| self.directory.join(name));
        fs::remove_dir_all(directory).unwrap();
    }

    pub fn grafbase_dev(&mut self) {
        let command = cmd!(
            cargo_bin("grafbase"),
            "--trace",
            "2",
            "--nohome",
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
        cmd!(
            cargo_bin("grafbase"),
            "--nohome",
            "dev",
            "--disable-watch",
            "--port",
            self.port.to_string()
        )
        .dir(&self.directory)
        .start()?
        .into_output()
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
        cmd!(cargo_bin("grafbase"), "--nohome", "reset")
            .dir(&self.directory)
            .run()
            .unwrap();
    }

    pub fn grafbase_dev_watch(&mut self) {
        let command = cmd!(
            cargo_bin("grafbase"),
            "--nohome",
            "dev",
            "--port",
            self.port.to_string()
        )
        .dir(&self.directory)
        .start()
        .unwrap();

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

impl Drop for Environment {
    fn drop(&mut self) {
        self.kill_processes();
        if let Some(dynamodb_client) = self.dynamodb_client.take() {
            tokio::runtime::Runtime::new().unwrap().block_on(async move {
                Self::delete_database_async(dynamodb_client, self.port).await;
            });
        }
    }
}
