#![allow(unused_crate_dependencies)]

use std::{path::PathBuf, str::FromStr};

use licensing::ES256KeyPair;

fn main() {
    let mut args = std::env::args();
    args.next();

    let key_pair = ES256KeyPair::generate();
    let private_key = key_pair.to_pem().unwrap();
    let public_key = key_pair.public_key().to_pem().unwrap();

    if let Some(path) = args.next() {
        let path = PathBuf::from_str(&path).unwrap();
        std::fs::write(path, &private_key).unwrap();
    };

    if let Some(path) = args.next() {
        let path = PathBuf::from_str(&path).unwrap();
        std::fs::write(path, &public_key).unwrap();
    }

    println!("private key:\n{}\n", private_key);
    println!("public key:\n{}", public_key);
}
