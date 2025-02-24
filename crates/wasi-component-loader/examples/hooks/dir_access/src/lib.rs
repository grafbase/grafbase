use grafbase_hooks::{Context, ErrorResponse, Headers, Hooks, grafbase_hooks};

struct Component;

#[grafbase_hooks]
impl Hooks for Component {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn on_gateway_request(&mut self, _: Context, _url: String, headers: Headers) -> Result<(), ErrorResponse> {
        match std::fs::read_to_string("./contents.txt") {
            Ok(contents) => headers.set("READ_CONTENTS", &contents).unwrap(),
            Err(e) => eprintln!("error reading file contents: {e}"),
        }

        if let Err(e) = std::fs::write("./guest_write.txt", "answer") {
            eprintln!("error writing file contents: {e}");
        }

        Ok(())
    }
}

grafbase_hooks::register_hooks!(Component);
