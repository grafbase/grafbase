#![allow(dead_code)]

use super::async_client::AsyncClient;
use super::kill_with_children::kill_with_children;
use super::{cargo_bin::cargo_bin, client::Client};
use backend::project::GraphType;
use common::consts::GRAFBASE_SCHEMA_FILE_NAME;
use duct::{cmd, Handle};
use std::io;
use std::path::Path;
use std::process::Output;
use std::sync::{Arc, Mutex};
use std::{env, fs, io::Write, path::PathBuf};
use tempfile::{tempdir, TempDir};

pub struct Environment {
    pub endpoint: String,
    pub playground_endpoint: String,
    pub directory_path: PathBuf,
    pub port: u16,
    temp_dir: Arc<TempDir>,
    schema_path: PathBuf,
    commands: CommandHandles,
    home: Option<PathBuf>,
    ts_config_dependencies_prepared: bool,
}

const DOT_ENV_FILE: &str = ".env";

pub fn get_free_port() -> u16 {
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
        Self::init_internal("./", get_free_port())
    }

    pub fn init_in_subdirectory(subdirectory_path: impl AsRef<Path>) -> Self {
        Self::init_internal(subdirectory_path, get_free_port())
    }

    pub async fn init_async() -> Self {
        Self::init_internal("./", get_free_port())
    }

    fn init_internal(subdirectory_path: impl AsRef<Path>, port: u16) -> Self {
        let temp_dir = tempdir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        let schema_path = temp_dir.path().join(subdirectory_path).join(GRAFBASE_SCHEMA_FILE_NAME);
        let directory_path = temp_dir.path().to_owned();
        println!("Using temporary directory {:?}", directory_path.as_os_str());
        let endpoint = format!("http://127.0.0.1:{port}/graphql");
        let playground_endpoint = format!("http://127.0.0.1:{port}");
        let temp_dir = Arc::new(temp_dir);
        Self {
            endpoint,
            playground_endpoint,
            directory_path,
            port,
            temp_dir,
            schema_path,
            commands: CommandHandles::new(),
            home: None,
            ts_config_dependencies_prepared: false,
        }
    }

    /// Same environment but different port.
    pub fn from(other: &Environment) -> Self {
        let port = get_free_port();
        let temp_dir = other.temp_dir.clone();
        let endpoint = format!("http://127.0.0.1:{port}/graphql");
        let playground_endpoint = format!("http://127.0.0.1:{port}");

        Self {
            directory_path: other.directory_path.clone(),
            commands: CommandHandles::new(),
            endpoint,
            playground_endpoint,
            schema_path: other.schema_path.clone(),
            temp_dir,
            port,
            home: other.home.clone(),
            ts_config_dependencies_prepared: other.ts_config_dependencies_prepared,
        }
    }

    pub fn create_client_with_options(&self, options: super::client::ClientOptions) -> Client {
        Client::new(
            self.endpoint.clone(),
            self.playground_endpoint.clone(),
            options,
            self.commands.clone(),
        )
    }

    pub fn create_client(&self) -> Client {
        Client::new(
            self.endpoint.clone(),
            self.playground_endpoint.clone(),
            super::client::ClientOptions::default(),
            self.commands.clone(),
        )
    }

    pub fn create_async_client(&self) -> AsyncClient {
        AsyncClient::new(
            self.endpoint.clone(),
            self.playground_endpoint.clone(),
            self.commands.clone(),
        )
    }

    // TODO: change this to set_schema
    //
    pub fn write_schema(&self, schema: impl AsRef<str>) {
        let schema = format!("extend schema @introspection(enable: true)\n{}", schema.as_ref());
        self.write_schema_without_introspection(schema)
    }

    pub fn write_schema_without_introspection(&self, schema: impl AsRef<str>) {
        // TODO: this is temporary until we update all tests to use SDK
        let _ = fs::remove_file("grafbase.config.ts");
        let _ = fs::remove_file("grafbase/grafbase.config.ts");
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

    pub async fn write_resolver_async(&self, path: impl AsRef<Path>, contents: impl AsRef<str>) {
        self.write_file_async(Path::new("resolvers").join(path.as_ref()), contents)
            .await;
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

    pub async fn write_file_async(&self, path: impl AsRef<Path>, contents: impl AsRef<str>) {
        let target_path = self.schema_path.parent().unwrap().join(path.as_ref());
        tokio::fs::create_dir_all(target_path.parent().unwrap()).await.unwrap();
        tokio::fs::write(target_path, contents.as_ref()).await.unwrap();
    }

    #[track_caller]
    pub fn write_json_file_to_project(&self, path: impl AsRef<Path>, contents: &serde_json::Value) {
        let contents = serde_json::to_string_pretty(contents).unwrap();
        let target_path = self.directory_path.join(path.as_ref());
        fs::create_dir_all(target_path.parent().unwrap()).unwrap();
        fs::write(target_path, contents).unwrap();
    }

    #[track_caller]
    pub fn load_file_from_project(&self, path: impl AsRef<Path>) -> String {
        fs::read_to_string(self.directory_path.join(path.as_ref())).unwrap()
    }

    #[track_caller]
    pub fn grafbase_introspect(&self, url: &str, headers: &[&str]) -> Output {
        let mut args = vec!["introspect", url];

        for header in headers {
            args.push("--header");
            args.push(*header);
        }

        duct::cmd(cargo_bin("grafbase"), args)
            .dir(&self.directory_path)
            .stdout_capture()
            .stderr_capture()
            .unchecked()
            .run()
            .unwrap()
    }

    #[track_caller]
    pub fn grafbase_init(&self, graph_type: GraphType) {
        let current_directory_path = self.schema_path.parent().expect("must be defined");
        std::fs::create_dir_all(current_directory_path).unwrap();
        cmd!(cargo_bin("grafbase"), "--trace", "2", "init", "-g", graph_type.as_ref())
            .dir(current_directory_path)
            .run()
            .unwrap();
    }

    #[track_caller]
    pub fn grafbase_init_output(&self, graph_type: GraphType) -> Output {
        let current_directory_path = self.schema_path.parent().expect("must be defined");
        std::fs::create_dir_all(current_directory_path).unwrap();
        cmd!(cargo_bin("grafbase"), "--trace", "2", "init", "-g", graph_type.as_ref())
            .dir(current_directory_path)
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
        .dir(&self.directory_path)
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap()
    }

    pub fn grafbase_link_non_interactive(&self, project: &str) -> Output {
        cmd!(cargo_bin("grafbase"), "link", "--project", project)
            .dir(&self.directory_path)
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
        .dir(&self.directory_path)
        .run()
        .unwrap();
    }

    pub fn with_home(mut self, path: PathBuf) -> Self {
        fs::create_dir_all(self.directory_path.join(&path)).unwrap();
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
        .dir(&self.directory_path);
        let command = command.start().unwrap();

        self.commands.0.lock().unwrap().push(command);
    }

    pub fn grafbase_start(&mut self) {
        let command = cmd!(
            cargo_bin("grafbase"),
            "--trace",
            "2",
            "start",
            "--listen-address",
            &format!("127.0.0.1:{}", self.port),
        )
        .dir(&self.directory_path);
        let command = command.start().unwrap();

        self.commands.0.lock().unwrap().push(command);
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
        .dir(&self.directory_path)
        .start()
        .unwrap();

        self.commands.0.lock().unwrap().push(command);
    }

    pub fn grafbase_dev_output(&mut self) -> io::Result<Output> {
        let command = cmd!(
            cargo_bin("grafbase"),
            "dev",
            "--disable-watch",
            "--port",
            self.port.to_string()
        )
        .dir(&self.directory_path);

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

    pub fn grafbase_dev_watch(&mut self) {
        let command = cmd!(
            cargo_bin("grafbase"),
            "--trace",
            "2",
            "dev",
            "--port",
            self.port.to_string()
        )
        .dir(&self.directory_path);

        let command = command.start().unwrap();

        self.commands.0.lock().unwrap().push(command);
    }

    pub fn grafbase_publish_dev(&mut self, name: impl AsRef<str>, url: impl AsRef<str>) {
        let command = cmd!(
            cargo_bin("grafbase"),
            "--trace",
            "2",
            "publish",
            "--dev",
            "--dev-api-port",
            self.port.to_string(),
            "--name",
            name.as_ref(),
            "--url",
            url.as_ref()
        )
        .dir(&self.directory_path);

        command.run().unwrap();
    }

    pub fn append_to_schema(&self, contents: &'static str) {
        let mut file = fs::OpenOptions::new().append(true).open(&self.schema_path).unwrap();

        file.write_all(format!("\n{contents}").as_bytes()).unwrap();

        file.sync_all().unwrap();

        drop(file);
    }

    pub fn kill_processes(&mut self) {
        let commands = std::mem::take(&mut *self.commands.0.lock().unwrap());
        commands.iter().for_each(|command| {
            kill_with_children(*command.pids().first().unwrap());
        });
    }

    pub fn has_database_directory(&mut self) -> bool {
        fs::metadata(self.directory_path.join(".grafbase/database")).is_ok()
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        self.kill_processes();
    }
}

#[derive(Clone)]
pub struct CommandHandles(Arc<Mutex<Vec<Handle>>>);

impl CommandHandles {
    pub fn new() -> Self {
        CommandHandles(Arc::new(Mutex::new(vec![])))
    }

    pub fn push(&mut self, handle: Handle) {
        self.0.lock().unwrap().push(handle);
    }

    pub fn still_running(&self) -> bool {
        self.0
            .lock()
            .unwrap()
            .iter()
            .all(|handle| handle.try_wait().unwrap().is_none())
    }

    pub fn kill_all(&self) {
        for command in self.0.lock().unwrap().iter() {
            command.kill().unwrap();
        }
    }
}
