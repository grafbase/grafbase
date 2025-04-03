use std::{fs, process};

#[test]
fn codegen_tests() {
    let bin_path = env!("CARGO_BIN_EXE_protoc-gen-grafbase-extension");

    insta::glob!("codegen/**/*.proto", |path| {
        let tmp = tempfile::tempdir().unwrap();

        let mut cmd = process::Command::new("protoc");

        cmd.arg("--plugin")
            .arg(bin_path)
            .arg("--grafbase-extension_out")
            .arg(tmp.path())
            .arg("-I")
            .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/codegen"))
            .arg(path);

        let output = cmd.output().unwrap();

        assert!(output.status.success(), "Expected success, got {:#?}", output);

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
