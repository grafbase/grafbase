use crate::{
    errors::CliError,
    watercolor::{self, watercolor},
};
use backend::project::{ConfigType, Template};
use colored::Colorize;
use common::{consts::GRAFBASE_TS_CONFIG_FILE_NAME, types::ResolverMessageLevel};
use common::{
    consts::{GRAFBASE_DIRECTORY_NAME, GRAFBASE_SCHEMA_FILE_NAME, LOCALHOST},
    environment::Warning,
};
use std::path::Path;

/// reports to stdout that the server has started
pub fn cli_header() {
    let version = env!("CARGO_PKG_VERSION");
    // TODO: integrate this with watercolor
    println!("{}", format!("Grafbase CLI {version}\n").dimmed());
}

/// reports to stdout that the server has started
pub fn start_server(resolvers_reported: bool, port: u16, start_port: u16) {
    if resolvers_reported {
        println!();
    }

    if port != start_port {
        println!(
            "Port {} is unavailable, started on the closest available port",
            watercolor!("{start_port}", @BrightBlue)
        );
    }
    println!("📡 Listening on port {}\n", watercolor!("{port}", @BrightBlue));
    println!(
        "- Pathfinder: {}",
        watercolor!("http://{LOCALHOST}:{port}", @BrightBlue)
    );
    // TODO: use proper formatting here
    println!(
        "- Endpoint:   {}\n",
        watercolor!("http://{LOCALHOST}:{port}/graphql", @BrightBlue)
    );
}

pub fn project_created(name: Option<&str>, template: Template<'_>) {
    let slash = std::path::MAIN_SEPARATOR.to_string();

    let schema_file_name = match template {
        Template::FromDefault(ConfigType::TypeScript) => GRAFBASE_TS_CONFIG_FILE_NAME,
        _ => GRAFBASE_SCHEMA_FILE_NAME,
    };

    if let Some(name) = name {
        watercolor::output!(r#"✨ {name} was successfully initialized!"#, @BrightBlue);

        let schema_path = &[".", name, GRAFBASE_DIRECTORY_NAME, schema_file_name].join(&slash);

        println!(
            "The schema for your new project can be found at {}",
            watercolor!("{schema_path}", @BrightBlue)
        );
    } else {
        watercolor::output!(r#"✨ Your project was successfully set up for Grafbase!"#, @BrightBlue);

        let schema_path = &[".", GRAFBASE_DIRECTORY_NAME, schema_file_name].join(&slash);

        println!(
            "Your new schema can be found at {}",
            watercolor!("{schema_path}", @BrightBlue)
        );
    }

    if let Template::FromDefault(ConfigType::TypeScript) = template {
        println!(
            "We've added our SDK to your {}, make sure to install dependencies before continuing.",
            watercolor!("package.json", @BrightBlue)
        );
    }
}

/// reports an error to stderr
pub fn error(error: &CliError) {
    watercolor::output_error!("Error: {error}", @BrightRed);
    if let Some(hint) = error.to_hint() {
        watercolor::output_error!("Hint: {hint}", @BrightBlue);
    }
}

pub fn warnings(warnings: &[Warning]) {
    for warning in warnings {
        let msg = warning.message();

        watercolor::output!("Warning: {msg}", @BrightYellow);

        if let Some(hint) = warning.hint() {
            watercolor::output!("Hint: {hint}", @BrightBlue);
        }

        println!();
    }
}

pub fn goodbye() {
    watercolor::output!("\n👋 See you next time!", @BrightBlue);
}

pub fn start_resolver_build(resolver_name: &str) {
    println!(
        "{}  - compiling resolver '{resolver_name}'…",
        watercolor!("wait", @Blue)
    );
}

pub fn complete_resolver_build(resolver_name: &str, duration: std::time::Duration) {
    let formatted_duration = if duration < std::time::Duration::from_secs(1) {
        format!("{}ms", duration.as_millis())
    } else {
        format!("{:.1}s", duration.as_secs_f64())
    };
    println!(
        "{} - resolver '{resolver_name}' compiled successfully in {formatted_duration}",
        watercolor!("event", @Green)
    );
}

pub fn resolver_message(resolver_name: &str, message: &str, level: ResolverMessageLevel) {
    match level {
        ResolverMessageLevel::Debug => watercolor::output!("[{resolver_name}] {message}", @BrightBlack),
        ResolverMessageLevel::Error => watercolor::output!("[{resolver_name}] {message}", @Red),
        ResolverMessageLevel::Info => watercolor::output!("[{resolver_name}] {message}", @Cyan),
        ResolverMessageLevel::Warn => watercolor::output!("[{resolver_name}] {message}", @Yellow),
    }
}

pub fn reload<P: AsRef<Path>>(path: P) {
    println!(
        "🔄 Detected a change in {path}, reloading",
        path = path.as_ref().display()
    );
}

pub fn project_reset() {
    watercolor::output!(r#"✨ Successfully reset your project!"#, @BrightBlue);
    #[cfg(target_family = "unix")]
    watercolor::output!(r#"If you have a running 'grafbase dev' instance in this project, it will need to be restarted for this change to take effect"#, @BrightBlue);
}

pub fn login(url: &str) {
    println!(
        "Please continue by opening the following URL:\n{}\n",
        watercolor!("{url}", @BrightBlue)
    );
}

pub fn login_success() {
    watercolor::output!("\n\n✨ Successfully logged in!", @BrightBlue);
}

// TODO: better handling of spinner position to avoid this extra function
pub fn login_error(error: &CliError) {
    watercolor::output!("\n\nError: {error}", @BrightRed);
    if let Some(hint) = error.to_hint() {
        watercolor::output!("Hint: {hint}", @BrightBlue);
    }
}

pub fn logout() {
    watercolor::output!("✨ Successfully logged out!", @BrightBlue);
}

// TODO change this to a spinner that is removed on success
pub fn deploy() {
    watercolor::output!("🕒 Your project is being deployed...", @BrightBlue);
}

// TODO change this to a spinner that is removed on success
pub fn create() {
    watercolor::output!("🕒 Your project is being created...", @BrightBlue);
}

pub fn deploy_success() {
    watercolor::output!("\n✨ Your project was successfully deployed!", @BrightBlue);
}

pub fn linked(name: &str) {
    watercolor::output!("\n✨ Successfully linked your local project to {name}!", @BrightBlue);
}

pub fn unlinked() {
    watercolor::output!("✨ Successfully unlinked your project!", @BrightBlue);
}

pub fn create_success(name: &str, urls: &[String]) {
    watercolor::output!("\n✨ {name} was successfully created!\n", @BrightBlue);
    watercolor::output!("Endpoints:", @BrightBlue);
    for url in urls {
        watercolor::output!("- https://{url}", @BrightBlue);
    }
}
