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
    pub contracts: Option<ContractsType>,

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
    #[serde(skip_serializing_if = "Directives::is_empty", default)]
    pub directives: Directives,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct AuthenticationType {}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthorizationType {
    #[serde(skip_serializing_if = "Directives::is_empty", default)]
    pub directives: Directives,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub group_by: Option<Vec<extension::AuthorizationGroupBy>>,
}

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize)]
pub struct Directives {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub definition: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub link_urls: Vec<String>,
}

impl Directives {
    pub fn is_empty(&self) -> bool {
        self.names.is_none() && self.definition.is_none() && self.link_urls.is_empty()
    }
}

impl<'de> serde::Deserialize<'de> for Directives {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DirectivesVisitor;

        impl<'de> serde::de::Visitor<'de> for DirectivesVisitor {
            type Value = Directives;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an array of directive names or a map with names, definition, and link_urls")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut names = Vec::new();
                while let Some(name) = seq.next_element::<String>()? {
                    names.push(name);
                }
                Ok(Directives {
                    names: Some(names),
                    definition: None,
                    link_urls: Vec::new(),
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut names = None;
                let mut definition = None;
                let mut link_urls = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "names" => {
                            if names.is_some() {
                                return Err(serde::de::Error::duplicate_field("names"));
                            }
                            names = Some(map.next_value::<Vec<String>>()?);
                        }
                        "definition" => {
                            if definition.is_some() {
                                return Err(serde::de::Error::duplicate_field("definition"));
                            }
                            definition = Some(map.next_value::<String>()?);
                        }
                        "link_urls" => {
                            if link_urls.is_some() {
                                return Err(serde::de::Error::duplicate_field("link_urls"));
                            }
                            link_urls = Some(map.next_value::<Vec<String>>()?);
                        }
                        _ => {
                            return Err(serde::de::Error::unknown_field(
                                &key,
                                &["names", "definition", "link_urls"],
                            ));
                        }
                    }
                }

                Ok(Directives {
                    names,
                    definition,
                    link_urls: link_urls.unwrap_or_default(),
                })
            }
        }

        deserializer.deserialize_any(DirectivesVisitor)
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct ContractsType {
    #[serde(skip_serializing_if = "Directives::is_empty", default)]
    pub directives: Directives,
}

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
