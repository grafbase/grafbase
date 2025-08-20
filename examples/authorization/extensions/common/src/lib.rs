use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Token {
    pub current_user_id: u32,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct AuthContext<'a> {
    // if you use serde_json or similar, you have to use String or Cow<'a, str> here
    #[serde(borrow)]
    pub scopes: HashMap<&'a str, String>,
}
