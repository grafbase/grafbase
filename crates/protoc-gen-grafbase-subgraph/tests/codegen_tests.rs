#![allow(clippy::panic)]

use std::{fs, process};

#[test]
fn codegen_tests() {
    let bin_path = env!("CARGO_BIN_EXE_protoc-gen-grafbase-subgraph");

    if cfg!(windows) {
        eprintln!("Skipping these tests on Windows, since there is no official protoc binary release.");
        return;
    }

    insta::glob!("codegen/**/*.proto", |path| {
        let tmp = tempfile::tempdir().unwrap();

        let mut cmd = process::Command::new("protoc");

        cmd.arg("--plugin")
            .arg(bin_path)
            .arg("--grafbase-subgraph_out")
            .arg(tmp.path())
            .arg("-I")
            .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/codegen"))
            .arg(path)
            .stderr(process::Stdio::inherit());

        let output = cmd.output().unwrap();

        assert!(
            output.status.success(),
            "Expected success, got {}\n{}",
            output.status,
            std::str::from_utf8(&output.stderr).unwrap(),
        );

        for entry in walkdir::WalkDir::new(tmp.path()) {
            let entry = entry.unwrap();

            if !entry.file_type().is_file() {
                continue;
            }

            match entry.path().extension().map(|ext| ext.to_str().unwrap()) {
                Some("graphql") => insta::assert_snapshot!("graphql file", fs::read_to_string(entry.path()).unwrap()),
                _ => panic!("Unexpected file: {:?}", entry.path()),
            }
        }
    });
}
