use crate::{consts::CREDENTIALS_FILE, errors::BackendError};
use common::environment::get_user_dot_grafbase_path;
use std::fs;

pub fn logout() -> Result<(), BackendError> {
    let user_dot_grafbase_path = get_user_dot_grafbase_path().ok_or(BackendError::NotLoggedIn)?;

    let credentials_path = user_dot_grafbase_path.join(CREDENTIALS_FILE);

    if credentials_path.exists() {
        fs::remove_file(credentials_path).map_err(BackendError::DeleteCredentialsFile)?;
    } else {
        return Err(BackendError::NotLoggedIn);
    }

    Ok(())
}
