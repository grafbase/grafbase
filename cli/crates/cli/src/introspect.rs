use crate::{cli_input::IntrospectCommand, errors::CliError, output::report};
use std::io::{IsTerminal as _, Write};
use tokio::runtime::Runtime;

pub(crate) fn introspect(command: &IntrospectCommand) -> Result<(), CliError> {
    match (command.url(), command.dev) {
        (Some(url), _) => {
            let headers = command.headers().collect::<Vec<_>>();
            introspect_remote(url, &headers, command.no_color)
        }
        (None, true) => introspect_local(command.no_color),
        (None, false) => {
            eprintln!("Error: Either the --url or the --dev argument must be provided.");
            std::process::exit(1);
        }
    }
}

fn introspect_local(no_color: bool) -> Result<(), CliError> {
    match server::introspect_local().map_err(CliError::ServerError)? {
        server::IntrospectLocalOutput::Sdl(schema) => {
            print_introspected_schema(&schema, no_color);
        }
        server::IntrospectLocalOutput::EmptyFederated => {
            report::federated_schema_local_introspection_not_implemented();
        }
    }

    Ok(())
}

fn introspect_remote(url: &str, headers: &[(&str, &str)], no_color: bool) -> Result<(), CliError> {
    let operation = grafbase_graphql_introspection::introspect(url, headers);

    match Runtime::new().unwrap().block_on(operation) {
        Ok(result) => {
            print_introspected_schema(&result, no_color);
            Ok(())
        }
        Err(e) => Err(CliError::Introspection(e)),
    }
}

fn print_introspected_schema(sdl: &str, no_color: bool) {
    use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

    let mut stdout = std::io::stdout();

    // No highlighting when stdout is not a tty (likely a pipe) or when explicitly requested.
    if no_color || !std::io::stdout().is_terminal() || no_color_env_var() {
        stdout.write_all(sdl.as_bytes()).ok();
        return;
    }

    const GRAMMAR: &str = include_str!("introspect/graphql_grammar.yml");

    let mut builder = syntect::parsing::SyntaxSetBuilder::new();
    builder.add(
        syntect::parsing::SyntaxDefinition::load_from_str(GRAMMAR, false, Some("graphql"))
            .expect("Loading the bundled grammar"),
    );
    let syntax_set = builder.build();

    let graphql = syntax_set
        .find_syntax_by_extension("graphql")
        .expect("graphql syntax to be bundled");
    let theme_set = syntect::highlighting::ThemeSet::load_defaults();
    let theme = &theme_set.themes["Solarized (dark)"];

    let mut highlighter = syntect::easy::HighlightLines::new(graphql, theme);

    for line in LinesWithEndings::from(sdl) {
        let ranges = highlighter
            .highlight_line(line, &syntax_set)
            .expect("line to be highlightable");

        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);

        if stdout.write_all(escaped.as_bytes()).is_err() {
            return;
        }
    }
}

/// https://no-color.org/
fn no_color_env_var() -> bool {
    std::env::var("NO_COLOR").is_ok_and(|value| !value.is_empty())
}
