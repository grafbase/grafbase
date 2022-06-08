use crate::consts::{LOCALHOST, MAX_PORT};
use std::{net::TcpListener, ops::Range};

/// determines if a port or port range are available
#[must_use]
pub fn find_available_port(search: bool, start_port: u16) -> Option<u16> {
    if search {
        find_available_port_in_range(start_port..MAX_PORT)
    } else {
        TcpListener::bind((LOCALHOST, start_port)).is_ok().then(|| start_port)
    }
}

/// finds an available port within a range
#[must_use]
pub fn find_available_port_in_range(mut range: Range<u16>) -> Option<u16> {
    range.find(|port| TcpListener::bind((LOCALHOST, *port)).is_ok())
}
