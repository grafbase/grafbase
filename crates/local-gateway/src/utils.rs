use common::consts::{LOCALHOST, MAX_PORT};
use std::net::TcpListener;

/// determines if a port or port range are available
pub fn get_availble_port(search: bool, start_port: u16) -> Option<u16> {
    let max_port = if search { MAX_PORT } else { start_port };

    let mut port_range = start_port..max_port;
    port_range.find(|port| TcpListener::bind((LOCALHOST, *port)).is_ok())
}
