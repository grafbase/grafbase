use crate::errors::CliError;
use inquire::InquireError;
use std::process;

impl From<InquireError> for CliError {
    fn from(error: InquireError) -> Self {
        match error {
            InquireError::NotTTY => CliError::PromptNotTTY,
            InquireError::IO(error) => CliError::PromptIoError(error),
            // exit normally without panicking on ESC or CTRL+C
            InquireError::OperationCanceled | InquireError::OperationInterrupted => process::exit(0),
            InquireError::InvalidConfiguration(_) | InquireError::Custom(_) => unreachable!(),
        }
    }
}
