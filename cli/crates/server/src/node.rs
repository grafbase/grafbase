use std::process::Stdio;

use tokio::process::Command;
use version_compare::Version;
use which::which;

use crate::consts::MIN_NODE_VERSION;

#[derive(thiserror::Error, Debug, Clone)]
pub enum NodeError {
    /// returned if node is not in the user $PATH
    #[error("Node.js does not seem to be installed")]
    NodeInPath,

    /// returned if the installed version of node is unsupported
    #[error("Node.js version {0} is unsupported")]
    OutdatedNode(String, String),

    /// returned if the installed version of node could not be retreived
    #[error("Could not retrieve the installed version of Node.js")]
    CheckNodeVersion,
}

pub async fn validate_node() -> Result<(), NodeError> {
    trace!("validating Node.js version");
    trace!("minimal supported Node.js version: {}", MIN_NODE_VERSION);

    which("node").map_err(|_| NodeError::NodeInPath)?;

    let node_version_string = get_node_version_string().await?;

    trace!("installed node version: {}", node_version_string);

    let node_version = Version::from(&node_version_string).ok_or(NodeError::CheckNodeVersion)?;
    let min_version = Version::from(MIN_NODE_VERSION).expect("must be valid");

    if node_version >= min_version {
        Ok(())
    } else {
        Err(NodeError::OutdatedNode(
            node_version_string,
            MIN_NODE_VERSION.to_owned(),
        ))
    }
}

async fn get_node_version_string() -> Result<String, NodeError> {
    let output = Command::new("node")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| NodeError::CheckNodeVersion)?
        .wait_with_output()
        .await
        .map_err(|_| NodeError::CheckNodeVersion)?;

    let node_version_string = String::from_utf8_lossy(&output.stdout).trim().to_owned();

    Ok(node_version_string)
}
