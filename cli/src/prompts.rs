use crate::errors::CliError;
use inquire::InquireError;
use std::process;

pub fn handle_inquire_error(error: InquireError) -> CliError {
    match error {
        InquireError::NotTTY => CliError::PromptNotTTY,
        InquireError::IO(error) => CliError::PromptIoError(error),
        // exit normally without panicking on ESC or CTRL+C
        InquireError::OperationCanceled | InquireError::OperationInterrupted => process::exit(0),
        InquireError::InvalidConfiguration(_) | InquireError::Custom(_) => unreachable!(),
    }
}
