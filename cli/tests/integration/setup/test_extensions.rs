use std::path::{Path, PathBuf};

pub(crate) enum TestExtensions {
    Echo,
}

impl TestExtensions {
    fn src_path(&self) -> PathBuf {
        let dir_name = match self {
            TestExtensions::Echo => "echo_extension",
        };

        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/integration/data/")
            .join(dir_name)
    }

    pub(crate) fn build_dir_path(&self) -> PathBuf {
        self.src_path().join("build")
    }

    pub(crate) async fn build(&self) {
        let src_path = self.src_path();

        tokio::process::Command::new(crate::GRAFBASE_CLI_BIN_PATH)
            .arg("extension")
            .arg("build")
            .current_dir(&src_path)
            .status()
            .await
            .unwrap();
    }
}
