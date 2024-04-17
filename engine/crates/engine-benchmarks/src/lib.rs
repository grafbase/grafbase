#![allow(unused_crate_dependencies)]

use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
};

/// A process that runs a graphql-mock
pub struct MockProcess {
    child: Child,
    pub port: u16,
}

impl MockProcess {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut child = Command::new("cargo")
            .args(["run", "-p", "engine-benchmarks", "--bin", "run-mock"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // Comment this line out if you want to debug
            .spawn()
            .unwrap();

        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut lines = stdout.lines();
        let value = serde_json::from_str::<serde_json::Value>(&lines.next().unwrap().unwrap()).unwrap();

        let port = value["port"].as_number().unwrap().as_i64().unwrap() as u16;

        // Not sure if we need to do this, but spawn a background thread that
        // processes the rest of our piped stdin so we don't fill buffers
        std::thread::spawn(move || {
            lines.for_each(|_| ());
        });

        MockProcess { child, port }
    }
}

impl Drop for MockProcess {
    fn drop(&mut self) {
        if let Err(err) = self.child.kill() {
            eprintln!("Killing MockProcess failed: {err:?}");
        }
        self.child.wait().ok();
    }
}
