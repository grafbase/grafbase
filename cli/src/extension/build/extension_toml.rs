use extension::{EventFilter, EventType};
use semver::Version;
use serde::Deserializer;
use serde_valid::Validate;

#[derive(serde::Deserialize)]
pub struct ExtensionToml {
    pub extension: ExtensionTomlExtension,
    #[serde(default)]
    pub directives: ExtensionTomlDirectives,
    #[serde(default)]
    pub permissions: ExtensionTomlPermissions,
    #[serde(default)]
    pub hooks: ExtensionTomlHooks,
}

#[derive(Default, serde::Deserialize)]
pub struct ExtensionTomlHooks {
    #[serde(default, deserialize_with = "deserialize_event_filter")]
    pub events: Option<EventFilter>,
}

#[derive(Default, serde::Deserialize)]
pub struct ExtensionTomlDirectives {
    pub definitions: Option<String>,
    pub field_resolvers: Option<Vec<String>>,
    pub resolvers: Option<Vec<String>>,
    pub authorization: Option<Vec<String>>,
}

#[derive(Default, serde::Deserialize)]
pub struct ExtensionTomlPermissions {
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub stdout: bool,
    #[serde(default)]
    pub stderr: bool,
    #[serde(default)]
    pub environment_variables: bool,
}

#[derive(serde::Deserialize, Validate)]
pub struct ExtensionTomlExtension {
    #[validate(pattern = "^[a-z0-9-]+$")]
    pub name: String,
    pub version: Version,
    // backwards compatibility for now.
    #[serde(alias = "kind")]
    pub r#type: ExtensionType,
    pub description: String,
    pub homepage_url: Option<url::Url>,
    pub repository_url: Option<url::Url>,
    pub license: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionType {
    Resolver,
    Authentication,
    Authorization,
    SelectionSetResolver,
    Hooks,
}

fn deserialize_event_filter<'de, D>(deserializer: D) -> Result<Option<EventFilter>, D::Error>
where
    D: Deserializer<'de>,
{
    struct EventFilterVisitor;

    impl<'de> serde::de::Visitor<'de> for EventFilterVisitor {
        type Value = Option<EventFilter>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("expecting string \"*\", or an array of values")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if value == "*" {
                Ok(Some(EventFilter::All))
            } else {
                value
                    .parse()
                    .map_err(|err| E::custom(err))
                    .map(|value| Some(EventFilter::Types(vec![value])))
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
            Ok(Some(EventFilter::Types(array)))
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(self)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(EventFilterVisitor)
}
