fn main() {
    cynic_codegen::register_schema("grafbase")
        .from_sdl_file("src/api/graphql/api.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
