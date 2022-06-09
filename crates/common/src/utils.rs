use crate::{consts::MAX_PORT, types::LocalAddressType};
use std::{net::TcpListener, ops::Range};

/// determines if a port or port range are available
#[must_use]
pub fn find_available_port(search: bool, start_port: u16, local_address_type: LocalAddressType) -> Option<u16> {
    if search {
        find_available_port_in_range(start_port..MAX_PORT, local_address_type)
    } else {
        let local_address = local_address_type.to_ip_v4();
        TcpListener::bind((local_address, start_port))
            .is_ok()
            .then(|| start_port)
    }
}

/// finds an available port within a range
#[must_use]
pub fn find_available_port_in_range(mut range: Range<u16>, local_address_type: LocalAddressType) -> Option<u16> {
    let local_address = local_address_type.to_ip_v4();
    range.find(|port| TcpListener::bind((local_address, *port)).is_ok())
}
