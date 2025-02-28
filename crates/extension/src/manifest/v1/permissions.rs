use enumflags2::bitflags;

#[bitflags]
#[repr(u16)]
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum ExtensionPermission {
    #[serde(rename = "network")]
    Network = 1 << 0,
    #[serde(rename = "stdout")]
    Stdout = 1 << 1,
    #[serde(rename = "stderr")]
    Stderr = 1 << 2,
    #[serde(rename = "environment_variables")]
    EnvironmentVariables = 1 << 3,
}

impl AsRef<str> for ExtensionPermission {
    fn as_ref(&self) -> &str {
        match self {
            ExtensionPermission::Network => "network",
            ExtensionPermission::Stdout => "stdout",
            ExtensionPermission::Stderr => "stderr",
            ExtensionPermission::EnvironmentVariables => "environment_variables",
        }
    }
}

pub(super) mod serializing {
    use super::*;
    use enumflags2::BitFlags;
    use serde::de::{self, SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<S>(permissions: &BitFlags<ExtensionPermission>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut vec = Vec::new();

        // Add each permission to the vector if it's set
        if permissions.contains(ExtensionPermission::Network) {
            vec.push("network");
        }
        if permissions.contains(ExtensionPermission::Stdout) {
            vec.push("stdout");
        }
        if permissions.contains(ExtensionPermission::Stderr) {
            vec.push("stderr");
        }
        if permissions.contains(ExtensionPermission::EnvironmentVariables) {
            vec.push("environment_variables");
        }

        let mut seq = serializer.serialize_seq(Some(vec.len()))?;

        for item in vec {
            seq.serialize_element(item)?;
        }

        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BitFlags<ExtensionPermission>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PermissionsVisitor;

        impl<'de> Visitor<'de> for PermissionsVisitor {
            type Value = BitFlags<ExtensionPermission>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a sequence of permission strings")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut permissions = BitFlags::empty();

                while let Some(value) = seq.next_element::<String>()? {
                    match value.as_str() {
                        "network" => permissions |= ExtensionPermission::Network,
                        "stdout" => permissions |= ExtensionPermission::Stdout,
                        "stderr" => permissions |= ExtensionPermission::Stderr,
                        "environment_variables" => permissions |= ExtensionPermission::EnvironmentVariables,
                        _ => {
                            return Err(de::Error::unknown_variant(
                                &value,
                                &["network", "stdout", "stderr", "environment_variables"],
                            ));
                        }
                    }
                }

                Ok(permissions)
            }
        }

        deserializer.deserialize_seq(PermissionsVisitor)
    }
}
