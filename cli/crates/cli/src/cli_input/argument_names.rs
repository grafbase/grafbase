// TODO see if there's a way to do this automatically (https://github.com/clap-rs/clap/discussions/4921)
pub trait ArgumentNames {
    /// returns the argument names used in a specific invocation of the CLI
    fn argument_names(&self) -> Option<Vec<&'static str>>;
}

pub fn filter_existing_arguments(arguments: &[(bool, &'static str)]) -> Option<Vec<&'static str>> {
    let arguments = arguments
        .iter()
        .filter(|arguments| arguments.0)
        .map(|arguments| arguments.1)
        .collect::<Vec<_>>();
    if arguments.is_empty() {
        None
    } else {
        Some(arguments)
    }
}
