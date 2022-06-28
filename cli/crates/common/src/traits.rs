/// marks errors that can be converted to an exit code
pub trait ToExitCode {
    /// returns the appropriate exit code for a given error
    fn to_exit_code(&self) -> i32;
}
