use super::errors::LoginApiError;

pub enum LoginMessage {
    CallbackUrl(String),
    Done,
    Error(LoginApiError),
}

#[derive(Debug)]
pub(crate) struct Account {
    pub id: String,
    pub name: String,
    pub slug: String,
}
