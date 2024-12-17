use std::any::Any;

/// converts an unknown panic parameter from [`std::thread::JoinHandle`] `join` to an [`Option<String>`]
#[must_use]
pub fn get_thread_panic_message(parameter: &Box<dyn Any + Send>) -> Option<String> {
    let str_message = parameter.downcast_ref::<&'static str>();
    let string_message = parameter.downcast_ref::<String>();
    match (str_message, string_message) {
        (Some(&message), None) => Some(message.to_string()),
        (None, Some(message)) => Some(message.clone()),
        _ => None,
    }
}
