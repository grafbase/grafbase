use strum_macros::EnumString;

#[derive(Debug, PartialEq, EnumString)]
pub enum TransactionCanceledReason {
    ConditionalCheckFailed,
    None,
    #[strum(default)]
    Unknown(String),
}

pub fn transaction_cancelled_reasons(message: &str) -> Option<Vec<TransactionCanceledReason>> {
    message
        .strip_prefix("Transaction cancelled, please refer cancellation reasons for specific reasons ")
        // Left with e.g. `[ConditionalCheckFailed, ConditionalCheckFailed]`.
        .map(|reasons_string| {
            reasons_string
                .strip_prefix('[')
                .expect("must start with [")
                .strip_suffix(']')
                .expect("must end with ]")
        })
        // Left with e.g. `ConditionalCheckFailed, ConditionalCheckFailed`.
        .map(|reasons_string| {
            reasons_string
                .split(", ")
                .map(|s| s.parse().unwrap()) // Parsing always succeeds.
                .collect()
        })
}
