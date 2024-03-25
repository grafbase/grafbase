#![allow(unused_crate_dependencies)]

use std::{path::PathBuf, str::FromStr};

use base64::{engine::general_purpose, Engine};
use ring::{
    rand::SystemRandom,
    signature::{EcdsaKeyPair, KeyPair},
};

fn main() {
    let mut args = std::env::args();
    args.next();

    let rand = SystemRandom::new();
    let document = EcdsaKeyPair::generate_pkcs8(licensing::SIGNING_ALGORITHM, &rand).unwrap();
    let key_pair = EcdsaKeyPair::from_pkcs8(licensing::SIGNING_ALGORITHM, document.as_ref(), &rand).unwrap();

    if let Some(path) = args.next() {
        let path = PathBuf::from_str(&path).unwrap();
        std::fs::write(path, document.as_ref()).unwrap();
    };

    if let Some(path) = args.next() {
        let path = PathBuf::from_str(&path).unwrap();
        std::fs::write(path, key_pair.public_key().as_ref()).unwrap();
    }

    println!("private key:\n{}", general_purpose::STANDARD.encode(&document));
    println!();
    println!(
        "public key:\n{}",
        general_purpose::STANDARD.encode(key_pair.public_key())
    );
}
