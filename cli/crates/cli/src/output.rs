pub mod report {

    use std::path::Path;

    use crate::{
        errors::CliError,
        watercolor::{self, watercolor},
    };
    use backend::types::FileEventType;
    use colored::Colorize;
    use common::consts::LOCALHOST;

    /// reports to stdout that the server has started
    pub fn cli_header() {
        let version = env!("CARGO_PKG_VERSION");
        // TODO: integrate this with watercolor
        println!("{}", format!("Grafbase CLI {version}\n").dimmed());
    }

    /// reports to stdout that the server has started
    pub fn start_server(port: u16, start_port: u16) {
        if port != start_port {
            println!(
                "port {} is unavailable, started on the closest available port",
                watercolor!("{start_port}", @BrightBlue)
            );
        }
        println!("📡 listening on port {}\n", watercolor!("{port}", @BrightBlue));
        println!(
            "- playground: {}",
            watercolor!("http://{LOCALHOST}:{port}", @BrightBlue)
        );
        // TODO: use proper formatting here
        println!(
            "- endpoint:   {}\n",
            watercolor!("http://{LOCALHOST}:{port}/graphql", @BrightBlue)
        );
    }

    pub fn project_created(name: Option<&str>) {
        let slash = std::path::MAIN_SEPARATOR.to_string();
        if let Some(name) = name {
            watercolor::output!(r#"✨ {name} was successfully initialized!"#, @BrightBlue);

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

    pub fn reload<P: AsRef<Path>>(path: P, _file_event_type: FileEventType) {
        println!(
            "🔄 detected a change in {path}, reloading",
            path = path.as_ref().display()
        );
    }

    pub fn project_reset() {
        watercolor::output!(r#"✨ successfully reset your project!"#, @BrightBlue);
        #[cfg(target_family = "unix")]
        watercolor::output!(r#"if you have a running 'grafbase dev' instance in this project, it will need to be restarted for this change to take effect"#, @BrightBlue);
    }

    pub fn login(url: &str) {
        println!(
            "please continue by opening the following URL:\n{}\n",
            watercolor!("{url}", @BrightBlue)
        );
    }

    pub fn login_success() {
        watercolor::output_error!("\n\n✨ successfully logged in!", @BrightBlue);
    }

    // TODO: better handling of spinner position to avoid this extra function
    pub fn login_error(error: &CliError) {
        watercolor::output_error!("\n\nerror: {error}", @BrightRed);
        if let Some(hint) = error.to_hint() {
            watercolor::output_error!("hint: {hint}", @BrightBlue);
        }
    }

    pub fn logout() {
        watercolor::output_error!("✨ successfully logged out!", @BrightBlue);
    }

    pub fn created(name: &str, urls: &[String]) {
        watercolor::output!("\n✨ {name} was successfully created!\n", @BrightBlue);
        watercolor::output!("Endpoints:", @BrightBlue);
        for url in urls {
            watercolor::output!("- https://{url}", @BrightBlue);
        }
    }
}
