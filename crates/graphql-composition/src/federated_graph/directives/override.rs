use crate::federated_graph::{StringId, SubgraphId};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub enum OverrideLabel {
    Percent(u8),
    Unknown(String),
}

impl std::fmt::Display for OverrideLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OverrideLabel::Percent(percent) => {
                f.write_str("percent(")?;
                percent.fmt(f)?;
                f.write_str(")")
            }
            OverrideLabel::Unknown(unknown) => f.write_str(unknown),
        }
    }
}

impl std::str::FromStr for OverrideLabel {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(percent) = s
            .strip_prefix("percent(")
            .and_then(|suffix| suffix.strip_suffix(')'))
            .and_then(|percent| u8::from_str(percent).ok())
        {
            Ok(OverrideLabel::Percent(percent))
        } else {
            Ok(OverrideLabel::Unknown(s.to_owned()))
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, PartialOrd)]
pub enum OverrideSource {
    Subgraph(SubgraphId),
    Missing(StringId),
}
