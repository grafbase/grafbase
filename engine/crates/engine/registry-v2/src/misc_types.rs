//! Some imported types from the old registry.
//!
//! Could look at improving these sometime but for now I'm just copying them into here.

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::minify_variant_names(serialize = "minified", deserialize = "minified")]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq, Default)]
pub enum Deprecation {
    #[default]
    NoDeprecated,
    Deprecated {
        reason: Option<String>,
    },
}

impl Deprecation {
    pub fn is_deprecated(&self) -> bool {
        matches!(self, Deprecation::Deprecated { .. })
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            Deprecation::NoDeprecated => None,
            Deprecation::Deprecated { reason } => reason.as_deref(),
        }
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl, Deprecation)]
#[derive(Clone, Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
pub struct FederationProperties {
    pub provides: Option<String>,
    pub tags: Vec<String>,
    pub r#override: Option<String>,
    pub external: bool,
    pub shareable: bool,
    pub inaccessible: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Hash, PartialEq)]
pub struct FieldSet(pub Vec<Selection>);

impl FieldSet {
    pub fn new(selections: impl IntoIterator<Item = Selection>) -> Self {
        FieldSet(selections.into_iter().collect())
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Hash, PartialEq)]
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, ConstraintType)]
pub struct Selection {
    pub field: String,
    pub selections: Vec<Selection>,
}

#[derive(Default, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum ScalarParser {
    /// Do not parse scalars, instead match the [`serde_json::Value`] type directly to the relevant
    /// [`Value`] type.
    PassThrough,

    /// Parse the scalar based on a list of well-known formats, trying to match the value to one of
    /// the formats. If no match is found, an error is returned.
    ///
    /// See [`PossibleScalar`] for more details.
    #[default]
    BestEffort,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnionDiscriminators(pub Vec<(String, UnionDiscriminator)>);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UnionDiscriminator {
    /// If the named field is present then this is the correct variant
    FieldPresent(String),
    /// This is the correct variant if the given field has one of the provided values
    FieldHasValue(String, Vec<serde_json::Value>),
    /// This is the correct variant if the input is of a particular type
    IsAScalar(ScalarKind),
    /// Fallback on this type if no others match
    Fallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ScalarKind {
    String,
    Number,
    Boolean,
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
/// Headers we should send to a connectors downstream server
pub struct ConnectorHeaders(pub Vec<(String, ConnectorHeaderValue)>);

impl ConnectorHeaders {
    pub fn new(headers: impl IntoIterator<Item = (String, ConnectorHeaderValue)>) -> Self {
        ConnectorHeaders(headers.into_iter().collect())
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ConnectorHeaderValue {
    /// We should send a static value for this header
    Static(String),
    /// We should pull the value for this header from the named header in the incoming
    /// request
    Forward(String),
}
