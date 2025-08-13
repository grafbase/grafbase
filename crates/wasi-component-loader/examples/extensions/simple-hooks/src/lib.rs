use grafbase_sdk::{
    HooksExtension,
    types::{Configuration, Error},
};

#[derive(HooksExtension)]
struct SimpleHooks;

impl HooksExtension for SimpleHooks {
    fn new(_: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }
}
