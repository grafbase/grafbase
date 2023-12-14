use std::process::Stdio;

use tokio::process::Command;
use version_compare::Version;
use which::which;

use crate::{consts::MIN_NODE_VERSION, errors::ServerError};

pub async fn validate_node() -> Result<(), ServerError> {
    trace!("validating Node.js version");
    trace!("minimal supported Node.js version: {}", MIN_NODE_VERSION);

    which("node").map_err(|_| ServerError::NodeInPath)?;

    let node_version_string = get_node_version_string().await?;

    trace!("installed node version: {}", node_version_string);

    let node_version = Version::from(&node_version_string).ok_or(ServerError::CheckNodeVersion)?;
    let min_version = Version::from(MIN_NODE_VERSION).expect("must be valid");

    if node_version >= min_version {
        Ok(())
    } else {
        Err(ServerError::OutdatedNode(
            node_version_string,
            MIN_NODE_VERSION.to_owned(),
        ))
    }
}

async fn get_node_version_string() -> Result<String, ServerError> {
    let output = Command::new("node")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| ServerError::CheckNodeVersion)?
        .wait_with_output()
        .await
        .map_err(|_| ServerError::CheckNodeVersion)?;

    let node_version_string = String::from_utf8_lossy(&output.stdout).trim().to_owned();

    Ok(node_version_string)
}
