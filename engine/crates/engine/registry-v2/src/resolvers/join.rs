use engine_value::{argument_set::ArgumentSet, Name, Value};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
pub struct JoinResolver {
    pub fields: Vec<(String, ArgumentSet)>,
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
