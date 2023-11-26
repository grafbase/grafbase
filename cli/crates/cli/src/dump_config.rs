use crate::errors::CliError;

pub(super) fn dump_config() -> Result<(), CliError> {
    let config = server::dump_config(env!("CARGO_PKG_VERSION").to_owned()).map_err(CliError::ServerError)?;
    println!("{config}");
    Ok(())
}
