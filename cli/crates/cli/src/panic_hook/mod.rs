pub mod report;

use crate::watercolor;
use report::{Method, Report};
use std::borrow::Cow;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use watercolor::watercolor;

/// A convenient metadata struct that describes a crate
pub struct Metadata {
    /// The crate version
    pub version: Cow<'static, str>,
    /// The crate name
    pub name: Cow<'static, str>,
    /// The URL of the crate's website
    pub homepage: Cow<'static, str>,
    /// The URL of the crate's repository
    pub repository: Cow<'static, str>,
    /// A discord invite link
    pub discord: Cow<'static, str>,
}

/// `panic-hook` initialisation macro
///
/// ```
/// panic_hook!();
/// ```
#[macro_export]
macro_rules! panic_hook {
    () => {
        #[allow(unused_imports)]
        use $crate::panic_hook::{handle_dump, print_msg, Metadata};

        #[allow(unused_variables)]
        let meta = Metadata {
            version: env!("CARGO_PKG_VERSION").into(),
            name: "the Grafbase CLI".into(),
            homepage: env!("CARGO_PKG_HOMEPAGE").into(),
            repository: "https://github.com/grafbase/grafbase".into(),
            discord: "https://discord.gg/grafbase".into(),
        };

        #[cfg(not(debug_assertions))]
        match ::std::env::var("RUST_BACKTRACE") {
            Err(_) => {
                panic::set_hook(Box::new(move |info: &PanicInfo<'_>| {
                    let file_path = handle_dump(&meta, info);
                    print_msg(file_path, &meta);
                    process::exit(1); //
                }));
            }
            Ok(_) => {}
        }
    };
}

/// Utility function that prints a message to our human users
#[allow(dead_code)]
pub fn print_msg<P: AsRef<Path>>(file_path: Option<P>, metadata: &Metadata) {
    let Metadata {
        name,
        homepage,
        repository,
        discord,
        ..
    } = metadata;
    let file_path = match file_path {
        Some(file_path) => format!("{}", file_path.as_ref().display()),
        None => "<Failed to store file to disk>".to_string(),
    };

    watercolor::output_error!(
        indoc::indoc!{r##"
            Well, this is embarrassing, {} had a problem and crashed.

            We have generated a report file at {}.
            To help us address this issue, please consider submitting a GitHub issue or sending a message on Discord and including the report.

            - Homepage: {}
            - Repository: {}
            - Discord: {}

            Thank you!
        "##},
        name,
        watercolor!("{file_path}", @BrightBlue),
        watercolor!("{homepage}", @BrightBlue),
        watercolor!("{repository}", @BrightBlue),
        watercolor!("{discord}", @BrightBlue),
        @BrightRed
    );
}

/// Utility function which will handle dumping information to disk
#[must_use]
#[allow(dead_code)]
pub fn handle_dump(meta: &Metadata, panic_info: &std::panic::PanicHookInfo<'_>) -> Option<PathBuf> {
    let mut explanation = String::new();

    let message = match (
        panic_info.payload().downcast_ref::<&str>(),
        panic_info.payload().downcast_ref::<String>(),
    ) {
        (Some(message), _) => Some((*message).to_string()),
        (_, Some(message)) => Some((*message).to_string()),
        (None, None) => None,
    };

    let cause = match message {
        Some(message) => message,
        None => "Unknown".into(),
    };

    match panic_info.location() {
        Some(location) => {
            let _: Result<_, _> = writeln!(
                explanation,
                "Panic occurred in file '{}' at line {}",
                location.file(),
                location.line()
            );
        }
        None => explanation.push_str("Panic location unknown.\n"),
    }

    let report = Report::new(&meta.name, &meta.version, Method::Panic, explanation, cause);

    if let Ok(path) = report.persist() {
        Some(path)
    } else {
        eprintln!(
            "{}",
            report
                .serialize()
                .unwrap_or_else(|| "<Could not serialize report JSON>".to_string())
        );
        None
    }
}
