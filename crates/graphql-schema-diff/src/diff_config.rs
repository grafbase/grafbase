/// Options for configuring the behavior of [crate::diff_with_config()].
#[derive(Default)]
pub struct DiffConfig {
    pub(crate) additions_inside_type_definitions: bool,
}

impl DiffConfig {
    /// Emit [Change]s for added fields, directives, interface implementations inside of added types and interfaces, added values and directives inside of added enums, added members and directives inside of added unions, and added fields and directives inside of added input objects.
    pub fn with_additions_inside_type_definitions(mut self, verbose_additions: bool) -> Self {
        self.additions_inside_type_definitions = verbose_additions;
        self
    }
}
