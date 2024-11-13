#[derive(Debug, Default, serde::Deserialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ComplexityControlConfig {
    pub mode: Option<ComplexityControlMode>,
    pub limit: Option<usize>,
    pub list_size: Option<usize>,
}

#[derive(Debug, serde::Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComplexityControlMode {
    Measure,
    Enforce,
}
