pub mod report {

    use crate::{
        errors::CliError,
        watercolor::{self, watercolor},
    };
    use common::consts::LOCALHOST;

    /// reports to stdout that the dev server has started
    pub fn cli_header() {
        let version = env!("CARGO_PKG_VERSION");
        watercolor::output!("Grafbase CLI v{version}", @hex("4A9C6D"), @@BrightBlue);
    }

    /// reports to stdout that the dev server has started
    pub fn start_server(port: u16, start_port: u16) {
        if port != start_port {
            println!(
                "port {} is unavailable, started on the closest available port",
                watercolor!("{start_port}", @BrightBlue)
            );
        }
        println!(
            "📡 started dev server on {}",
            watercolor!("http://{LOCALHOST}:{port}", @BrightBlue)
        );
    }

    pub fn project_created(name: Option<&str>) {
        let slash = std::path::MAIN_SEPARATOR.to_string();
        if let Some(name) = name {
            watercolor::output!(r#"✨ "{name}" was succesfully created!"#, @BrightBlue);

            let schema_path = &[".", name, "grafbase", "schema.graphql"].join(&slash);

            println!(
                "the schema for your new project can be found at {}",
                watercolor!("{schema_path}", @BrightBlue)
            );
        } else {
            watercolor::output!(r#"✨ your project was successfully set up for Grafbase!"#, @BrightBlue);

            let schema_path = &[".", "grafbase", "schema.graphql"].join(&slash);

            println!(
                "your new schema can be found at {}",
                watercolor!("{schema_path}", @BrightBlue)
            );
        }
    }

    /// reports an error to stderr
    pub fn error(error: &CliError) {
        watercolor::output_error!("error: {error}", @BrightRed);
        if let Some(hint) = error.to_hint() {
            watercolor::output_error!("hint: {hint}", @BrightBlue);
        }
    }

    pub fn goodbye() {
        watercolor::output_error!("\n👋 see you next time!", @BrightBlue);
    }
}
