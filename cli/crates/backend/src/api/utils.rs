use super::consts::PROJECT_METADATA_FILE;
use common::environment::Environment;

pub fn project_linked() -> bool {
    let environment = Environment::get();
    environment
        .project_dot_grafbase_path
        .join(PROJECT_METADATA_FILE)
        .exists()
}
