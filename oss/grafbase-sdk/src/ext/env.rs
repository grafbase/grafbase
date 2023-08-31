use worker::Result;

use crate::sys::kv::KvStore;

pub trait EnvExt {
    fn kv(&self, binding: &str) -> Result<KvStore>;
}

impl EnvExt for worker::Env {
    fn kv(&self, binding: &str) -> Result<KvStore> {
        self.get_binding::<KvStore>(binding)
    }
}
