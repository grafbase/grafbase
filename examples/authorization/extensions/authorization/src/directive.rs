#[derive(serde::Deserialize)]
pub struct JwtScopeArguments<'a> {
    #[serde(borrow)]
    pub scopes: Vec<&'a str>,
}

#[derive(serde::Deserialize)]
pub struct AccessControlArguments<'a> {
    #[serde(borrow, default)]
    pub arguments: Option<Arguments<'a>>,
    #[serde(borrow, default)]
    pub fields: Option<Fields<'a>>,
}

#[serde_with::serde_as]
#[derive(serde::Deserialize)]
pub struct Arguments<'a> {
    #[serde(borrow, flatten)]
    #[serde_as(as = "serde_with::Map<_, _>")]
    map: Vec<(&'a str, serde_json::Value)>,
}

impl Arguments<'_> {
    // We assume that only field is provided and we expect it to be a u32 ID
    pub fn id_as_u32(&self) -> u32 {
        self.map.first().and_then(|(_, value)| value.as_u64()).unwrap() as u32
    }
}

#[serde_with::serde_as]
#[derive(serde::Deserialize)]
pub struct Fields<'a> {
    #[serde(borrow, flatten)]
    #[serde_as(as = "serde_with::Map<_, _>")]
    map: Vec<(&'a str, serde_json::Value)>,
}

impl Fields<'_> {
    // We assume that only field is provided and we expect it to be a u32 ID
    pub fn id_as_u32(&self) -> u32 {
        self.map.first().and_then(|(_, value)| value.as_u64()).unwrap() as u32
    }
}
