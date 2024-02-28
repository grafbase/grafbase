#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationLimitsConfig {
    /// Limits the deepest nesting of selection sets in an operation,
    /// including fields in fragments.
    pub depth: u16,
    /// Limits the number of unique fields included in an operation,
    /// including fields of fragments. If a particular field is included
    /// multiple times via aliases, it's counted only once.
    pub height: u16,
    /// Limits the total number of aliased fields in an operation,
    /// including fields of fragments.
    pub aliases: u16,
    /// Limits the number of root fields in an operation, including root
    /// fields in fragments. If a particular root field is included multiple
    /// times via aliases, each usage is counted.
    pub root_fields: u16,
    /// Query complexity takes the number of fields as well as the depth and
    /// any pagination arguments into account. Every scalar field adds 1 point,
    /// every nested field adds 2 points, and every pagination argument multiplies
    /// the nested objects score by the number of records fetched.
    pub complexity: u16,
}
