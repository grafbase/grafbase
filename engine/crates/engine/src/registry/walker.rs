pub struct OutputTypeWalker<'a> {
    registry: &'a Registry,
    ty: &'a OutputType,
}

impl OutputTypeWalker {
    pub fn field(&self, name: &str) -> OutputTypeWalker {}
}
