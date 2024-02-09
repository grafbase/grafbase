use engine_value::{argument_set::ArgumentSet, Name, Value};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
pub struct JoinResolver {
    pub fields: Vec<(String, ArgumentSet)>,
}

// ArgumentSet can't be hashed so we've got a manual impl here that goes via JSON.
// Would be nice to get rid of the Hash requirement from these types
impl core::hash::Hash for JoinResolver {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (name, arguments) in &self.fields {
            name.hash(state);
            serde_json::to_string(&arguments).unwrap_or_default().hash(state);
        }
    }
}

impl JoinResolver {
    pub fn new(fields: impl IntoIterator<Item = (String, Vec<(Name, Value)>)>) -> Self {
        JoinResolver {
            fields: fields
                .into_iter()
                .map(|(name, arguments)| (name, ArgumentSet::new(arguments)))
                .collect(),
        }
    }
}
