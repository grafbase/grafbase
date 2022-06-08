use std::net::Ipv4Addr;

pub const DB_FILE: &str = "data.sqlite";
pub const DB_URL: &str = "sqlite:data.sqlite";
pub const CREATE_TABLE: &str = include_str!("../sql/create-table.sql");
pub const LOCALHOST_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
