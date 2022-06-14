mod report;

use report::{Method, Report};
use std::borrow::Cow;
use std::panic::PanicInfo;
use std::path::{Path, PathBuf};

/// A convenient metadata struct that describes a crate
pub struct Metadata {
    /// The crate version
    pub version: Cow<'static, str>,
    /// The crate name
    pub name: Cow<'static, str>,
    /// The URL of the crate's website
    pub homepage: Cow<'static, str>,
}

/// `panic-hook` initialisation macro
///
/// ```
/// panic_hook::setup!();
/// ```
#[macro_export]
macro_rules! setup {
    () => {
        #[allow(unused_imports)]
        use std::panic::{self, PanicInfo};
        #[allow(unused_imports)]
        use $crate::{handle_dump, print_msg, Metadata};

        let meta = Metadata {
            version: env!("CARGO_PKG_VERSION").into(),
            name: "Grafbase CLI".into(),
            homepage: env!("CARGO_PKG_HOMEPAGE").into(),
        };

        #[cfg(not(debug_assertions))]
        match ::std::env::var("RUST_BACKTRACE") {
            Err(_) => {
                panic::set_hook(Box::new(move |info: &PanicInfo| {
                    let file_path = handle_dump(&meta, info);
                    print_msg(file_path, &meta);
                }));
            }
            Ok(_) => {}
        }
    };
}

/// Utility function that prints a message to our human users
pub fn print_msg<P: AsRef<Path>>(file_path: Option<P>, metadata: &Metadata) {
    let Metadata { name, homepage, .. } = metadata;
    let file_path = match file_path {
        Some(file_path) => format!("{}", file_path.as_ref().display()),
        None => "<Failed to store file to disk>".to_string(),
    };

    eprintln!(
        indoc::indoc! {r##"
            Well, this is embarrassing.
            {name} had a problem and crashed.
            
            We have generated a report file at "{file_path}". 
            To help us address this issue, please consider submitting a GitHub issue or sending us an email with the subject of "{name} Crash Report" and include the report as an attachment.

            - Homepage: {homepage}

            Thank you!
        "##},
        name = name,
        file_path = file_path,
        homepage = homepage
    );
}

/// Utility function which will handle dumping information to disk
#[must_use]
pub fn handle_dump(meta: &Metadata, panic_info: &PanicInfo<'_>) -> Option<PathBuf> {
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
        Some(location) => explanation.push_str(&format!(
            "Panic occurred in file '{}' at line {}\n",
            location.file(),
            location.line()
        )),
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
                .unwrap_or_else(|| "<Could not serialize report TOML>".to_string())
        );
        None
    }
}
