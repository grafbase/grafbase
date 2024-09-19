pub fn cached_tests() -> Vec<CachedTest> {
    let Ok(data) = std::fs::read_to_string("tests.json") else {
        return vec![];
    };

    serde_json::from_str(&data).unwrap()
}

/// A cached test definition as stored in tests.json
///
/// This is used to speed up startup of our test harness
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub struct CachedTest {
    pub(crate) suite: String,
    pub(crate) index: usize,
}

impl CachedTest {
    pub fn new(suite: &str, index: usize) -> Self {
        CachedTest {
            suite: suite.into(),
            index,
        }
    }

    pub fn name(&self) -> String {
        format!("{}::{}", self.suite, self.index)
    }
}
