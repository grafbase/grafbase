pub(crate) fn compose_after_addition_success(subgraph_name: &str) {
    eprintln!("🧩 Successfully composed schema after adding subgraph {subgraph_name}");
}

pub(crate) fn compose_after_addition_failure(subgraph_name: &str) {
    eprintln!("❌ Failed to compose schema after adding subgraph {subgraph_name}");
}

pub(crate) fn compose_after_removal_success(subgraph_name: &str) {
    eprintln!("🧩 Successfully composed schema after removing subgraph {subgraph_name}");
}

pub(crate) fn compose_after_removal_failure(subgraph_name: &str, errors: &str) {
    eprintln!("❌ Failed to compose schema after removing subgraph {subgraph_name}. Errors:\n{errors}");
}
