#![allow(unused_crate_dependencies)]

use std::{path::PathBuf, str::FromStr};

use chrono::Utc;
use licensing::{ES256KeyPair, License};
use ulid::Ulid;

#[allow(clippy::panic)]
fn main() {
    let mut args = std::env::args();
    args.next();

    let private_key = match args.next() {
        Some(path) => {
            let pem = std::fs::read_to_string(path).unwrap();
            ES256KeyPair::from_pem(&pem).unwrap()
        }
        None => panic!("please give path to a private key"),
    };

    let license = License {
        graph_id: Ulid::new(),
        account_id: Ulid::new(),
    };

    let token = license.sign(&private_key, Utc::now()).unwrap();

    match args.next() {
        Some(path) => {
            let path = PathBuf::from_str(&path).unwrap();
            std::fs::write(path, &token).unwrap();
        }
        None => panic!("please give a path to store the license"),
    }

    println!("{token}");
}
