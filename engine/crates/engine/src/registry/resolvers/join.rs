use engine_value::{argument_set::ArgumentSet, Name, Value};

// ArgumentSet can't be hashed so we've got a manual impl here that goes via JSON.
// Would be nice to get rid of the Hash requirement from these types
#[cfg(delete_me_maybe)]
impl core::hash::Hash for JoinResolver {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (name, arguments) in &self.fields {
            name.hash(state);
            serde_json::to_string(&arguments).unwrap_or_default().hash(state);
        }
    }
}
