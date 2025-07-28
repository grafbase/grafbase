use extension::{EventFilter, EventType};
use semver::Version;
use serde_valid::Validate;
use url::Url;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionToml {
    pub extension: Definition,
    #[serde(default)]
    pub permissions: Permissions,

    // == type specific ==
    #[serde(default)]
    pub resolver: Option<ResolverType>,
    #[allow(unused)]
    #[serde(default)]
    pub authentication: Option<AuthenticationType>,
    #[serde(default)]
    pub authorization: Option<AuthorizationType>,
    #[serde(default)]
    pub hooks: Option<HooksType>,
    #[allow(unused)]
    #[serde(default)]
    pub cntracts: Option<ContractsType>,

    // == LEGACY ==
    #[serde(default, rename = "directives")]
    pub legacy_directives: LegacyDirectives,
}

//
// === extension definition ===
//

#[derive(serde::Deserialize, Validate)]
pub struct Definition {
    #[validate(pattern = "^[a-z0-9-]+$")]
    pub name: String,
    pub version: Version,
    // backwards compatibility for now.
    #[serde(alias = "kind")]
    pub r#type: ExtensionType,
    pub description: String,
    pub homepage_url: Option<Url>,
    pub repository_url: Option<Url>,
    pub license: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionType {
    Resolver,
    Authentication,
    Authorization,
    SelectionSetResolver,
    Hooks,
    Contracts,
}

//
// === extension permissions ===
//

#[derive(Default, serde::Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub stdout: bool,
    #[serde(default)]
    pub stderr: bool,
    #[serde(default)]
    pub environment_variables: bool,
}

//
// === extension types ===
//

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ResolverType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directives: Option<Vec<String>>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct AuthenticationType {}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthorizationType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directives: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<Vec<extension::AuthorizationGroupBy>>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct ContractsType {}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct HooksType {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<EventFilterWrapper>,
}

#[derive(Clone)]
pub struct EventFilterWrapper(pub EventFilter);

impl From<EventFilterWrapper> for EventFilter {
    fn from(wrapper: EventFilterWrapper) -> Self {
        wrapper.0
    }
}

impl serde::Serialize for EventFilterWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            EventFilter::All => serializer.serialize_str("*"),
            EventFilter::Types(types) => types.serialize(serializer),
        }
    }
}
impl<'de> serde::Deserialize<'de> for EventFilterWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EventFilterVisitor;

        impl<'de> serde::de::Visitor<'de> for EventFilterVisitor {
            type Value = EventFilter;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("expecting string \"*\", or an array of values")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value == "*" {
                    Ok(EventFilter::All)
                } else {
                    value
                        .parse()
                        .map_err(|err| E::custom(err))
                        .map(|value| EventFilter::Types(vec![value]))
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut array = Vec::new();
                while let Some(value) = seq.next_element::<EventType>()? {
                    array.push(value);
                }
                Ok(EventFilter::Types(array))
            }
        }

        deserializer.deserialize_any(EventFilterVisitor).map(Self)
    }
}

//
// === legacy ===
//

#[derive(Default, serde::Deserialize)]
pub struct LegacyDirectives {
    pub definitions: Option<String>,
    pub field_resolvers: Option<Vec<String>>,
    pub resolvers: Option<Vec<String>>,
    pub authorization: Option<Vec<String>>,
}
