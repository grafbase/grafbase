use crate::output::report;
use crate::CliError;
use common::consts::DEFAULT_PORT;
use local_gateway::dev_server_api::start_dev_server;

/// cli wrapper for [`local_gateway::dev_server_api::start_dev_server`]
///
/// # Errors
///
/// returns [`CliError::LocalGatewayError`] if the the local gateway returns an error
///
/// returns [`CliError::DevServerPanic`] if the development server panics
pub fn dev(search: bool, external_port: Option<u16>) -> Result<(), CliError> {
    trace!("attempting to start dev server");
    let start_port = external_port.unwrap_or(DEFAULT_PORT);
    let server_handle = match start_dev_server(external_port, search) {
        Ok((port, handle)) => {
            report::start_server(port, start_port);
            handle
        }
        Err(error) => return Err(CliError::LocalGatewayError(error)),
    };

    server_handle
        .join()
        .map_err(|panic_parameter| match panic_parameter.downcast_ref::<&'static str>() {
            Some(&parameter) => CliError::DevServerPanic(parameter.to_string()),
            None => match panic_parameter.downcast_ref::<String>() {
                Some(parameter) => CliError::DevServerPanic(parameter.clone()),
                None => CliError::DevServerPanic("unknown error".to_owned()),
            },
        })?
        .map_err(|error| {
            report::spawned_thread_error(&error.to_string());
            CliError::DevServerPanic("miniflare error".to_owned())
        })?;

    Ok(())
}
