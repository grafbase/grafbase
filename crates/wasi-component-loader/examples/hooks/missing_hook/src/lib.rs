use grafbase_hooks::{Hooks, grafbase_hooks};

struct Component;

#[grafbase_hooks]
impl Hooks for Component {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }
}

grafbase_hooks::register_hooks!(Component);
