#![allow(dead_code)]

use super::kill_with_children::kill_with_children;
use super::{cargo_bin::cargo_bin, client::Client};
use duct::{cmd, Handle};
use std::{env, fs, io::Write, path::PathBuf, process::Command};
use tempfile::{tempdir, TempDir};

pub struct Environment {
    pub endpoint: String,
    directory: PathBuf,
    temp_dir: TempDir,
    schema_path: PathBuf,
    commands: Vec<Handle>,
    port: u16,
}

impl Environment {
    pub fn init(port: u16) -> Self {
        let temp_dir = tempdir().unwrap();
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

    pub fn create_client(&self) -> Client {
        Client::new(self.endpoint.clone())
    }

    pub fn write_schema(&self, schema: &'static str) {
        fs::write(&self.schema_path, schema).unwrap();
    }

    pub fn grafbase_init(&self) {
        Command::new(cargo_bin("grafbase"))
            .args(&["init"])
            .current_dir(&self.directory)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    pub fn grafbase_dev(&mut self) {
        let command = cmd!(
            cargo_bin("grafbase"),
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
}

impl Drop for Environment {
    fn drop(&mut self) {
        self.commands.iter().for_each(|command| {
            kill_with_children(*command.pids().first().unwrap());
        })
    }
}
