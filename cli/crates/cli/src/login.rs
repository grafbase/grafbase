use crate::{errors::CliError, output::report};
use backend::api::{login, types::LoginMessage};
use common::utils::get_thread_panic_message;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    sync::mpsc::{channel, RecvTimeoutError},
    thread::{sleep, spawn},
    time::Duration,
};

pub fn login() -> Result<(), CliError> {
    let (message_sender, message_receiver) = channel();

    let login_handle = spawn(|| login::login(message_sender).map_err(CliError::BackendApiError));

    if let Ok(LoginMessage::CallbackUrl(url)) = message_receiver.recv() {
        report::login(&url);
        // sleeping 1 second to prevent the browser opening immediately
        sleep(Duration::from_secs(1));
        // as we show the URL in the CLI output, not being able
        // to open the browser needs no handling
        let _ = webbrowser::open(&url);
    };

    let spinner = ProgressBar::new_spinner()
        .with_message("waiting for authentication to be completed")
        .with_style(
            ProgressStyle::with_template("{spinner} {wide_msg:.dim}")
                .expect("must parse")
                .tick_chars("🕛🕐🕑🕒🕓🕔🕕🕖🕗🕘🕙🕚✅"),
        );

    loop {
        match message_receiver.recv_timeout(Duration::from_millis(250)) {
            Ok(LoginMessage::Done) => {
                spinner.finish_with_message("token received");
                report::login_success();
            }
            Ok(LoginMessage::Error(error)) => {
                spinner.finish_with_message("token received");
                report::login_error(&CliError::LoginApiError(error));
            }
            Err(error) => match error {
                RecvTimeoutError::Timeout => spinner.inc(1),
                RecvTimeoutError::Disconnected => break,
            },
            _ => {}
        }
    }

    login_handle
        .join()
        .map_err(|parameter| match get_thread_panic_message(&parameter) {
            Some(message) => CliError::LoginPanic(message),
            None => CliError::LoginPanic("unknown error".to_owned()),
        })?
}
