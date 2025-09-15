fn main() {
    let proto_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../proto");

    tonic_build::configure()
        .type_attribute("routeguide.Point", "#[derive(Hash, Eq)]")
        .compile_protos(
            &[&format!("{proto_path}/route_guide.proto")],
            &[proto_path],
        )
        .unwrap();
}
