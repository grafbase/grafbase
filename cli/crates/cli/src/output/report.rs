use crate::{
    cli_input::LogLevelFilters,
    errors::CliError,
    logs::LogEvent,
    watercolor::{self, watercolor},
};
use backend::{
    project::ConfigType,
    types::{NestedRequestScopedMessage, RequestCompletedOutcome},
};
use colored::Colorize;
use common::consts::GRAFBASE_TS_CONFIG_FILE_NAME;
use common::types::{LogLevel, UdfKind};
use common::{
    consts::{GRAFBASE_DIRECTORY_NAME, GRAFBASE_SCHEMA_FILE_NAME, LOCALHOST},
    environment::Warning,
};
use std::{net::IpAddr, path::Path};

/// reports to stdout that the server has started
pub fn cli_header() {
    let version = env!("CARGO_PKG_VERSION");
    // TODO: integrate this with watercolor
    println!("{}", format!("Grafbase CLI {version}\n").dimmed());
}

pub fn start_prod_server(ip: IpAddr, port: u16) {
    println!("📡 Listening on {}\n", watercolor!("{ip}:{port}", @BrightBlue));
}
/// reports to stdout that the server has started
pub fn start_dev_server(resolvers_reported: bool, port: u16, start_port: u16) {
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

pub fn project_created(name: Option<&str>, config_type: ConfigType) {
    let slash = std::path::MAIN_SEPARATOR.to_string();

    let schema_file_name = match config_type {
        ConfigType::TypeScript => GRAFBASE_TS_CONFIG_FILE_NAME,
        ConfigType::GraphQL => GRAFBASE_SCHEMA_FILE_NAME,
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
            "Your new configuration can be found at {}",
            watercolor!("{schema_path}", @BrightBlue)
        );
    }

    if let ConfigType::TypeScript = config_type {
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

pub fn start_udf_build_all() {
    println!("{} compiling user defined functions...", watercolor!("wait", @Cyan),);
}

pub fn start_udf_build(udf_kind: UdfKind, udf_name: &str) {
    println!(
        "{} compiling {udf_kind} {udf_name}...",
        watercolor!("wait", @Cyan),
        udf_name = udf_name.to_string().bold()
    );
}

pub fn complete_udf_build_all(duration: std::time::Duration) {
    let formatted_duration = if duration < std::time::Duration::from_secs(1) {
        format!("{}ms", duration.as_millis())
    } else {
        format!("{:.1}s", duration.as_secs_f64())
    };
    println!(
        "{} user defined functions compiled successfully in {formatted_duration}",
        watercolor!("event", @BrightMagenta),
    );
}

pub fn complete_udf_build(udf_kind: UdfKind, udf_name: &str, duration: std::time::Duration) {
    let formatted_duration = if duration < std::time::Duration::from_secs(1) {
        format!("{}ms", duration.as_millis())
    } else {
        format!("{:.1}s", duration.as_secs_f64())
    };
    println!(
        "{} compiled {udf_kind} {udf_name} successfully in {formatted_duration}",
        watercolor!("event", @BrightMagenta),
        udf_name = udf_name.to_string().bold()
    );
}

#[allow(clippy::needless_pass_by_value)]
fn format_response_body(indent: &str, body: Option<String>, content_type: Option<String>) -> Option<String> {
    use itertools::Itertools;
    body.and_then(|body| match content_type.as_deref() {
        Some("application/json") => serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|value| serde_json::to_string_pretty(&value).ok()),
        Some("text/plain") => Some(body),
        other => {
            trace!("unsupported content type for tracing the body: {other:?}");
            None
        }
    })
    .map(|formatted_body| {
        formatted_body
            .lines()
            .map(|line| format!("{indent}{indent}{line}"))
            .join("\n")
    })
}

pub fn operation_log(
    name: Option<String>,
    duration: std::time::Duration,
    request_completed: RequestCompletedOutcome,
    nested_events: Vec<NestedRequestScopedMessage>,
    log_level_filters: LogLevelFilters,
) {
    if !log_level_filters.graphql_operations.should_display(LogLevel::Info) {
        return;
    }

    let (name, r#type, colour, duration) = match request_completed {
        RequestCompletedOutcome::Success { r#type } => {
            let colour = match r#type {
                common::types::OperationType::Query { is_introspection } => {
                    if is_introspection && !log_level_filters.graphql_operations.should_display(LogLevel::Debug) {
                        return;
                    }
                    watercolor::colored::Color::Green
                }
                common::types::OperationType::Mutation => watercolor::colored::Color::Green,
                common::types::OperationType::Subscription => {
                    return;
                }
            };
            (name, Some(r#type), colour, duration)
        }
        RequestCompletedOutcome::BadRequest => (name, None, watercolor::colored::Color::Red, duration),
    };

    let formatted_duration = format_duration(duration);

    let formatted_name = name
        .map(|name| format!(" {}", name.to_string().bold()))
        .unwrap_or_default();

    let formatted_type = r#type.map_or_else(|| "operation".to_owned(), |value| value.to_string());

    println!(
        "{formatted_type}{formatted_name} {formatted_duration}",
        formatted_type = formatted_type.color(colour)
    );

    log_nested_events(nested_events, log_level_filters);
}

fn log_nested_events(nested_events: Vec<NestedRequestScopedMessage>, log_level_filters: LogLevelFilters) {
    let indent = "  ";

    for nested_event in nested_events {
        match nested_event {
            NestedRequestScopedMessage::UdfMessage {
                udf_kind,
                udf_name,
                level,
                message,
            } => {
                if !log_level_filters.functions.should_display(level) {
                    continue;
                }

                let message_colour = match level {
                    LogLevel::Debug => watercolor::colored::Color::BrightBlack,
                    LogLevel::Error => watercolor::colored::Color::Red,
                    LogLevel::Info => watercolor::colored::Color::Cyan,
                    LogLevel::Warn => watercolor::colored::Color::Yellow,
                };

                println!(
                    "{indent}{} {} {}",
                    watercolor!("{udf_kind}", @Blue),
                    udf_name.bold(),
                    message.to_string().color(message_colour)
                );
            }
            NestedRequestScopedMessage::NestedRequest {
                url,
                method,
                status_code,
                duration,
                body,
                content_type,
            } => {
                let required_log_level = if status_code >= 400 {
                    LogLevel::Error
                } else {
                    LogLevel::Info
                };

                if !log_level_filters.fetch_requests.should_display(required_log_level) {
                    continue;
                }

                // A minor presentational tweak for URLs.
                let url: url::Url = url.parse().expect("must be a valid URL surely");
                let mut url_string = url.to_string();

                if url.path() == "/" && url.query().is_none() {
                    url_string = url_string.trim_end_matches('/').to_owned();
                }

                let formatted_duration = format_duration(duration);

                println!(
                    "{indent}{} {} {} {status_code} {formatted_duration}",
                    watercolor!("fetch", @Yellow),
                    method.bold(),
                    url_string.bold(),
                );

                if log_level_filters.fetch_requests.should_display(LogLevel::Debug) {
                    if let Some(formatted_body) = format_response_body(indent, body, content_type) {
                        println!("{formatted_body}");
                    }
                }
            }
            NestedRequestScopedMessage::SqlQuery {
                successful,
                sql,
                duration,
                body,
            } => {
                let required_log_level = if successful { LogLevel::Debug } else { LogLevel::Error };

                if !log_level_filters.fetch_requests.should_display(required_log_level) {
                    continue;
                }

                let formatted_duration = format_duration(duration);

                let status = if successful {
                    watercolor!("OK", @Green)
                } else {
                    watercolor!("ERROR", @Red)
                };

                println!(
                    "{indent}{} {} {status} {formatted_duration}",
                    watercolor!("sql", @Yellow),
                    sql.bold(),
                );

                if log_level_filters.fetch_requests.should_display(LogLevel::Debug) {
                    let content_type = Some(String::from("application/json"));

                    if let Some(formatted_body) = format_response_body(indent, body, content_type) {
                        println!("{formatted_body}");
                    }
                }
            }
        }
    }
}

pub fn format_duration(duration: std::time::Duration) -> String {
    [
        ("ns", duration.as_nanos()),
        ("μs", duration.as_micros()),
        ("ms", duration.as_millis()),
    ]
    .into_iter()
    .find(|(_, value)| *value < 1000)
    .map_or_else(
        || format!("{:.2}s", duration.as_secs_f64()),
        |(suffix, value)| format!("{value}{suffix}"),
    )
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

pub fn linked_non_interactive() {
    watercolor::output!("✨ Successfully linked your local project!", @BrightBlue);
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

pub fn print_log_entry(
    LogEvent {
        created_at,
        message,
        log_event_type,
        ..
    }: LogEvent,
) {
    let created_at: chrono::DateTime<chrono::Local> = chrono::DateTime::from(created_at);

    let rest = match log_event_type {
        crate::logs::LogEventType::Request { duration, .. } => {
            format!("{message} {duration}", duration = format_duration(duration))
        }
        crate::logs::LogEventType::FunctionMessage {
            log_level,
            function_kind,
            function_name,
        } => format!("[{log_level}] {function_kind} {function_name} | {message}"),
    };
    println!("{} {rest}", created_at.to_rfc3339());
}
