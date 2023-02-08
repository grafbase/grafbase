use crate::{consts::CREDENTIALS_FILE, errors::BackendError};
use common::environment::get_user_dot_grafbase_path;
use std::fs;

pub fn logout() -> Result<(), BackendError> {
    let user_dot_grafbase_path = get_user_dot_grafbase_path().ok_or(BackendError::NotLoggedIn)?;

    let credentials_path = user_dot_grafbase_path.join(CREDENTIALS_FILE);

    match credentials_path.try_exists() {
        Ok(true) => fs::remove_file(credentials_path).map_err(BackendError::DeleteCredentialsFile),
        Ok(false) => Err(BackendError::NotLoggedIn),
        Err(error) => Err(BackendError::ReadCredentialsFile(error)),
    }
}
