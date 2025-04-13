mod display_utils;
mod render_graphql_sdl;
mod schema;
mod translate_schema;

use prost::Message as _;
use prost_types::compiler::{CodeGeneratorRequest, CodeGeneratorResponse, code_generator_response::File};
use render_graphql_sdl::render_graphql_sdl;
use std::{
    env,
    io::{self, Read as _, Write as _},
    process,
};
use translate_schema::translate_schema;

fn bail(error: String) -> CodeGeneratorResponse {
    CodeGeneratorResponse {
        error: Some(error),
        supported_features: None,
        file: vec![],
    }
}

fn generate(raw_request: &[u8]) -> CodeGeneratorResponse {
    let request = match CodeGeneratorRequest::decode(raw_request) {
        Ok(request) => request,
        Err(decode_err) => {
            return bail(format!(
                "Failed to decode CodeGeneratorRequest from stdin: {decode_err}",
            ));
        }
    };

    let translated_schema = translate_schema(request);

    let mut graphql_schema = String::new();

    render_graphql_sdl(&translated_schema, &mut graphql_schema).unwrap();

    CodeGeneratorResponse {
        error: None,
        supported_features: None,
        file: vec![File {
            name: Some("schema.graphql".to_owned()),
            insertion_point: None,
            generated_code_info: None,
            content: Some(graphql_schema),
        }],
    }
}

fn main() -> io::Result<()> {
    if env::args().any(|x| x == "--version") {
        println!(env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;

    let response = generate(&buf);

    buf.clear();

    response.encode(&mut buf).expect("error encoding response");

    io::stdout().write_all(&buf)?;

    Ok(())
}
