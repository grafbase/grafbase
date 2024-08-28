use std::ops::Deref;
use std::str::FromStr;

use ulid::Ulid;

use federated_server::GdnResponse;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GdnResponseMock(GdnResponse);

impl Deref for GdnResponseMock {
    type Target = GdnResponse;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl GdnResponseMock {
    pub fn mock(sdl: &str) -> GdnResponseMock {
        GdnResponseMock(GdnResponse {
            account_id: Ulid::from_str("01HR7NP3A4NDVWC10PZW6ZMC5P").unwrap(),
            graph_id: Ulid::from_str("01HR7NPB8E3YW29S5PPSY1AQKR").unwrap(),
            branch: "main".to_string(),
            branch_id: Ulid::from_str("01HR7NPB8E3YW29S5PPSY1AQKA").unwrap(),
            sdl: sdl.to_string(),
            version_id: Ulid::from_str("01HR7NPYWWM6DEKACKKN3EPFP2").unwrap(),
        })
    }

    pub fn as_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}
