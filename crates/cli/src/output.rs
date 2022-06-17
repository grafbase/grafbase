pub mod report {
    use crate::errors::CliError;
    use colorize::{colorize, Color};
    use common::consts::LOCALHOST;

    /// reports to stdout that the dev server has started
    pub fn cli_header() {
        let version = env!("CARGO_PKG_VERSION");
        colorize::println!("Grafbase CLI v{}", version, hex("4A9C6D"), Color::BrightBlue);
    }

    /// reports to stdout that the dev server has started
    pub fn start_server(port: u16, start_port: u16) {
        if port != start_port {
            println!(
                "port {} is unavailable, started on the closest available port",
                colorize!("{}", start_port, Color::BrightBlue)
            );
        }
        println!(
            "📡 started dev server on {}",
            colorize!("http://{LOCALHOST}:{}", port, Color::BrightBlue)
        );
    }

    /// reports an error to stderr
    pub fn error(error: &CliError) {
        colorize::eprintln!("error: {}", error, Color::BrightRed);
        if let Some(hint) = error.to_hint() {
            colorize::eprintln!("hint: {}", hint, Color::BrightBlue);
        }
    }

    pub fn goodbye() {
        colorize::eprintln!("{}", "\n👋 see you next time!", Color::BrightBlue);
    }
}
