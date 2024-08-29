use std::sync::{Arc, Mutex};

use duct::Handle;

#[derive(Clone)]
pub struct CommandHandles(Arc<Mutex<Vec<Handle>>>);

impl CommandHandles {
    pub fn new() -> Self {
        CommandHandles(Arc::new(Mutex::new(vec![])))
    }

    pub fn push(&mut self, handle: Handle) {
        self.0.lock().unwrap().push(handle);
    }

    pub fn still_running(&self) -> bool {
        self.0
            .lock()
            .unwrap()
            .iter()
            .all(|handle| handle.try_wait().unwrap().is_none())
    }

    pub fn kill_all(&self) {
        for command in self.0.lock().unwrap().iter() {
            command.kill().unwrap();
        }
    }
}

impl Default for CommandHandles {
    fn default() -> Self {
        Self::new()
    }
}
