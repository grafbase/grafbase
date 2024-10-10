// TODO see if there's a way to do this automatically (https://github.com/clap-rs/clap/discussions/4921)
pub trait ArgumentNames {
    /// returns the argument names used in a specific invocation of the CLI
    fn argument_names(&self) -> Option<Vec<&'static str>>;
}
