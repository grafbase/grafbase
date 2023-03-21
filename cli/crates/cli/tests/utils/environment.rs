#![allow(dead_code)]

use super::async_client::AsyncClient;
use super::kill_with_children::kill_with_children;
use super::{cargo_bin::cargo_bin, client::Client};
use duct::{cmd, Handle};
use std::collections::HashMap;
use std::io;
use std::process::Output;
use std::sync::Arc;
use std::{env, fs, io::Write, path::PathBuf};
use tempfile::{tempdir, TempDir};

pub struct Environment {
    pub endpoint: String,
    pub directory: PathBuf,
    temp_dir: Arc<TempDir>,
    schema_path: PathBuf,
    commands: Vec<Handle>,
    port: u16,
}

const DOT_ENV_FILE: &str = ".env";

impl Environment {
    pub fn init(port: u16) -> Self {
        let temp_dir = Arc::new(tempdir().unwrap());
        env::set_current_dir(temp_dir.path()).unwrap();

        let schema_path = temp_dir.path().join("grafbase").join("schema.graphql");

        Self {
            directory: temp_dir.path().to_owned(),
            commands: vec![],
            endpoint: format!("http://127.0.0.1:{port}/graphql"),
            schema_path,
            temp_dir,
            port,
        }
    }

    pub fn from(other: &Environment, port: u16) -> Self {
        let temp_dir = other.temp_dir.clone();

        Self {
            directory: other.directory.clone(),
            commands: vec![],
            endpoint: format!("http://127.0.0.1:{port}/graphql"),
            schema_path: other.schema_path.clone(),
            temp_dir,
            port,
        }
    }

    pub fn create_client(&self) -> Client {
        Client::new(self.endpoint.clone())
    }

    pub fn create_async_client(&self) -> AsyncClient {
        AsyncClient::new(self.endpoint.clone())
    }

    pub fn write_schema(&self, schema: &'static str) {
        fs::write(&self.schema_path, schema).unwrap();
    }

    pub fn grafbase_init(&self) {
        cmd!(cargo_bin("grafbase"), "init").dir(&self.directory).run().unwrap();
    }

    pub fn grafbase_init_output(&self) -> Output {
        cmd!(cargo_bin("grafbase"), "init")
            .dir(&self.directory)
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
        .dir(&self.directory)
        .start()
        .unwrap();

        self.commands.push(command);
    }

    pub fn grafbase_dev_output(&mut self) -> io::Result<Output> {
        cmd!(
            cargo_bin("grafbase"),
            "dev",
            "--disable-watch",
            "--port",
            self.port.to_string()
        )
        .dir(&self.directory)
        .start()?
        .into_output()
    }

    pub fn set_variables(&mut self, variables: HashMap<String, String>) {
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
        let command = cmd!(cargo_bin("grafbase"), "dev", "--port", self.port.to_string())
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

    pub fn has_dot_grafbase_directory(&mut self) -> bool {
        fs::metadata(self.directory.clone().join(".grafbase")).is_ok()
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        self.kill_processes();
    }
}
