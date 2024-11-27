use grafbase_hooks::{grafbase_hooks, Hooks};

struct Component;

/// Start overriding the default hook implementations in this trait implementation.
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
