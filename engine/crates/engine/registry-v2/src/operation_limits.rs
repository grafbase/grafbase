#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OperationLimits {
    pub depth: Option<u16>,
    pub height: Option<u16>,
    pub aliases: Option<u16>,
    pub root_fields: Option<u16>,
    pub complexity: Option<u16>,
}

impl OperationLimits {
    pub fn any_enabled(&self) -> bool {
        *self != Default::default()
    }
}
