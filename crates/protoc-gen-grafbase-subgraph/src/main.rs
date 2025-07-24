mod display_utils;
mod render_graphql_sdl;
mod schema;
mod translate_schema;

use protobuf::plugin::{CodeGeneratorRequest, CodeGeneratorResponse, code_generator_response::File};
use protobuf::{CodedOutputStream, Message};
use render_graphql_sdl::render_graphql_sdl;
use std::{
    env,
    io::{self, Read as _, Write as _},
    process,
};
use translate_schema::translate_schema;

fn bail(error: String) -> CodeGeneratorResponse {
    let mut response = CodeGeneratorResponse::new();
    response.error = Some(error);
    response
}

fn generate(raw_request: &[u8]) -> CodeGeneratorResponse {
    let request = match CodeGeneratorRequest::parse_from_bytes(raw_request) {
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

    let mut response = CodeGeneratorResponse::new();
    let mut file = File::new();
    file.set_name("schema.graphql".to_owned());
    file.set_content(graphql_schema);
    response.file.push(file);
    response
}

fn main() -> io::Result<()> {
    if env::args().any(|x| x == "--version") {
        println!(env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;

    let response = generate(&buf);

    let mut output_buf = Vec::new();
    {
        let mut output_stream = CodedOutputStream::vec(&mut output_buf);
        response.write_to(&mut output_stream).expect("error encoding response");
        output_stream.flush().expect("error flushing response");
    }

    io::stdout().write_all(&output_buf)?;

    Ok(())
}
