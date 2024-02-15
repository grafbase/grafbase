#[derive(thiserror::Error, Debug)]
pub enum InputValueSerdeError {
    #[error("{0}")]
    Message(String),
}

impl serde::de::Error for InputValueSerdeError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        InputValueSerdeError::Message(msg.to_string())
    }
}
