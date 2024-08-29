use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

pub fn listen_address() -> SocketAddr {
    let port = get_free_port();
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port))
}

fn get_free_port() -> u16 {
    const INITIAL_PORT: u16 = 14712;

    let test_state_directory_path = std::env::temp_dir().join("grafbase/cli-tests");
    std::fs::create_dir_all(&test_state_directory_path).unwrap();
    let lock_file_path = test_state_directory_path.join("port-number.lock");
    let port_number_file_path = test_state_directory_path.join("port-number.txt");
    let mut lock_file = fslock::LockFile::open(&lock_file_path).unwrap();
    lock_file.lock().unwrap();
    let port_number = if port_number_file_path.exists() {
        std::fs::read_to_string(&port_number_file_path)
            .unwrap()
            .trim()
            .parse::<u16>()
            .unwrap()
            + 1
    } else {
        INITIAL_PORT
    };
    std::fs::write(&port_number_file_path, port_number.to_string()).unwrap();
    lock_file.unlock().unwrap();
    port_number
}
