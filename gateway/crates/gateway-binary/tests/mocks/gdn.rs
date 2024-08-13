use std::ops::Deref;
use std::str::FromStr;

use ulid::Ulid;

use federated_server::GraphMetadata;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct GdnResponseMock(GraphMetadata);
impl Deref for GdnResponseMock {
    type Target = GraphMetadata;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl GdnResponseMock {
    pub(crate) fn mock(sdl: &str) -> GdnResponseMock {
        GdnResponseMock(GraphMetadata {
            account_id: Ulid::from_str("01HR7NP3A4NDVWC10PZW6ZMC5P").unwrap(),
            graph_id: Ulid::from_str("01HR7NPB8E3YW29S5PPSY1AQKR").unwrap(),
            branch: "main".to_string(),
            branch_id: Ulid::from_str("01HR7NPB8E3YW29S5PPSY1AQKA").unwrap(),
            sdl: sdl.to_string(),
            version_id: "01HR7NPYWWM6DEKACKKN3EPFP2".to_string(),
        })
    }

    pub(crate) fn as_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}
