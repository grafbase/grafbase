use super::errors::ApiError;
use common::consts::CREDENTIALS_FILE;
use common::environment::Environment;
use std::fs;

/// Deletes the login credentials file
///
/// # Errors
///
/// - returns [`BackendError::NotLoggedIn`] if the user is not logged in when attempting to log out
///
/// - returns [`BackendError::DeleteCredentialsFile`] if ~/.grafbase/credentials.json could not be deleted
///
/// - returns [`BackendError::ReadCredentialsFile`] if ~/.grafbase/credentials.json could not be read
pub fn logout() -> Result<(), ApiError> {
    let environment = Environment::get();

    let credentials_path = environment.user_dot_grafbase_path.join(CREDENTIALS_FILE);

    match credentials_path.try_exists() {
        Ok(true) => fs::remove_file(credentials_path).map_err(ApiError::DeleteCredentialsFile),
        Ok(false) => Err(ApiError::NotLoggedIn),
        Err(error) => Err(ApiError::ReadCredentialsFile(error)),
    }
}
