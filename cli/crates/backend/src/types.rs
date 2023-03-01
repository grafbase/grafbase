use crate::errors::LoginApiError;
pub use server::types::ServerMessage;

pub enum LoginMessage {
    CallbackUrl(String),
    Done,
    Error(LoginApiError),
}
