pub mod report {
    use crate::errors::CliError;
    use colored::Colorize;
    use common::consts::LOCALHOST;
    use std::io;

    /// reports to stdout that the dev server has started
    pub fn start_server(port: u16, start_port: u16) {
        if port != start_port {
            println!(
                "port {} is unavailable, started on the closest available port",
                format!("{}", start_port).bright_blue()
            )
        }
        println!(
            "started dev server on {}",
            format!("http://{LOCALHOST}:{}", port).bright_blue()
        );
    }

    /// reports an error to stderr
    pub fn error(error: &CliError) {
        eprintln!("{}", format!("error: {}", error).bright_red());
        if let Some(hint) = error.to_hint() {
            eprintln!("{}", format!("hint: {}", hint).bright_blue());
        }
    }

    /// reports an error to stderr
    pub fn miniflare_error(error: io::Error) {
        eprintln!("{}", error);
    }
}
