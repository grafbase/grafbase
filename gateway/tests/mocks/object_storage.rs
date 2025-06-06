use std::ops::Deref;
use std::str::FromStr;

use ulid::Ulid;

use federated_server::ObjectStorageResponse;

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct ObjectStorageResponseMock(ObjectStorageResponse);
impl Deref for ObjectStorageResponseMock {
    type Target = ObjectStorageResponse;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectStorageResponseMock {
    pub(crate) fn mock(sdl: &str) -> ObjectStorageResponseMock {
        ObjectStorageResponseMock(ObjectStorageResponse {
            account_id: Ulid::from_str("01HR7NP3A4NDVWC10PZW6ZMC5P").unwrap(),
            graph_id: Ulid::from_str("01HR7NPB8E3YW29S5PPSY1AQKR").unwrap(),
            branch: "main".to_string(),
            branch_id: Ulid::from_str("01HR7NPB8E3YW29S5PPSY1AQKA").unwrap(),
            sdl: sdl.to_string(),
            version_id: Ulid::from_str("01HR7NPYWWM6DEKACKKN3EPFP2").unwrap(),
        })
    }

    pub(crate) fn as_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}
