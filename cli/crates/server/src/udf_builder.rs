use crate::consts::{ENTRYPOINT_SCRIPT_FILE_NAME, KV_DIR_NAME};
use crate::errors::UdfBuildError;
use common::environment::Environment;
use common::types::UdfKind;
use std::path::PathBuf;

async fn extract_udf_wrapper_worker_contents(udf_kind: UdfKind) -> Result<String, UdfBuildError> {
    trace!("extracting {udf_kind} wrapper worker contents");
    let environment = Environment::get();
    tokio::fs::read_to_string(
        environment
            .user_dot_grafbase_path
            .join(crate::consts::WRAPPER_WORKER_JS_PATH),
    )
    .await
    .map_err(|err| UdfBuildError::ExtractUdfWrapperWorkerContents(udf_kind, err))
}

const UDF_EXTENSIONS: [&str; 2] = ["js", "ts"];

#[allow(clippy::too_many_lines)]
pub(crate) async fn build(
    environment: &Environment,
    environment_variables: &std::collections::HashMap<String, String>,
    udf_kind: UdfKind,
    udf_name: &str,
    _tracing: bool,
) -> Result<PathBuf, UdfBuildError> {
    use path_slash::PathBufExt as _;

    let project = environment.project.as_ref().expect("must be present");

    // FIXME: that's a hack, need to change the wrapper script to only check the final part of the
    // URL.
    let udf_wrapper_worker_contents = extract_udf_wrapper_worker_contents(udf_kind)
        .await?
        .replace("\"/invoke\"", &format!("\"{}\"", udf_url_path(udf_kind, udf_name)));

    trace!("building {udf_kind} '{udf_name}'");

    let udf_input_file_path_without_extension = project.udfs_source_path(udf_kind).join(udf_name);
    let udf_build_artifact_directory_path = project.udfs_build_artifact_path(udf_kind).join(udf_name);

    let mut udf_input_file_path = None;
    for extension in UDF_EXTENSIONS {
        let possible_udf_input_file_path = udf_input_file_path_without_extension.with_extension(extension);
        if tokio::fs::try_exists(&possible_udf_input_file_path)
            .await
            .expect("must succeed")
        {
            udf_input_file_path = Some(possible_udf_input_file_path);
        }
    }
    let udf_input_file_path = udf_input_file_path
        .ok_or_else(|| UdfBuildError::UdfDoesNotExist(udf_kind, udf_input_file_path_without_extension.clone()))?;

    let udf_wrapper_worker_contents = udf_wrapper_worker_contents
        .replace(
            "${UDF_MAIN_FILE_PATH}",
            udf_input_file_path.to_slash().expect("must be valid UTF-8").as_ref(),
        )
        .replace(
            "${UDF_KV_DIR_PATH}",
            project
                .dot_grafbase_directory_path
                .join(KV_DIR_NAME)
                .to_slash()
                .expect("must be valid UTF-8")
                .as_ref(),
        );

    tokio::fs::create_dir_all(&udf_build_artifact_directory_path)
        .await
        .map_err(|_err| UdfBuildError::CreateDir(udf_build_artifact_directory_path.clone(), udf_kind))?;

    let udf_build_package_json_path = udf_build_artifact_directory_path.join("package.json");

    let package_json = serde_json::json!({
        "main": ENTRYPOINT_SCRIPT_FILE_NAME,
    });
    let new_package_json_contents = serde_json::to_string_pretty(&package_json).expect("must be valid JSON");
    trace!("new package.json contents:\n{new_package_json_contents}");

    tokio::fs::write(&udf_build_package_json_path, new_package_json_contents)
        .await
        .map_err(|err| UdfBuildError::CreateUdfArtifactFile(udf_build_package_json_path.clone(), udf_kind, err))?;

    let dist_path = udf_build_package_json_path
        .parent()
        .expect("must have parent")
        .join("dist");

    tokio::fs::create_dir_all(&dist_path)
        .await
        .map_err(|_| UdfBuildError::CreateDir(dist_path.clone(), udf_kind))?;

    let process_env_prelude = format!(
        "globalThis.process.env = {};",
        serde_json::to_string(&environment_variables).expect("must be valid JSON")
    );

    let content_with_env = format!("{process_env_prelude}\n{udf_wrapper_worker_contents}");

    let entrypoint_js_path = dist_path.join(ENTRYPOINT_SCRIPT_FILE_NAME);

    tokio::fs::write(&entrypoint_js_path, content_with_env)
        .await
        .map_err(|err| UdfBuildError::CreateUdfArtifactFile(entrypoint_js_path.clone(), udf_kind, err))?;

    Ok(udf_build_package_json_path)
}

pub(crate) fn udf_url_path(kind: UdfKind, name: &str) -> String {
    format!("/{}/{}/invoke", kind.to_string().to_lowercase(), slug::slugify(name))
}
