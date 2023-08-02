mod grouping;
mod v2;
mod v3;
mod v3_1;
mod version;

use std::collections::HashMap;

use petgraph::{dot::Dot, graph::NodeIndex, Graph};
use url::Url;

use self::version::OpenApiVersion;
use crate::{
    graph::{Edge, Node},
    Error, Format,
};

pub fn parse(data: String, format: Format) -> Result<ParseOutput, Vec<Error>> {
    let version = from_str::<OpenApiVersion>(&data, format)?;

    match version {
        OpenApiVersion::V2 => {
            let spec = from_str(&data, format)?;
            drop(data);
            v2::parse(spec).try_into()
        }
        OpenApiVersion::V3 => {
            let spec = from_str(&data, format)?;
            drop(data);
            v3::parse(spec).try_into()
        }
        OpenApiVersion::V3_1 => {
            let spec = from_str(&data, format)?;
            drop(data);
            let result: ParseOutput = v3_1::parse(spec).try_into()?;
            // println!("{:?}", result.dot());
            Ok(result)
        }
        OpenApiVersion::Unknown(version) => Err(vec![Error::UnsupportedVersion(version)]),
    }
}

pub struct ParseOutput {
    pub graph: Graph<Node, Edge>,
    pub url: Result<Url, Error>,
}

impl ParseOutput {
    #[allow(dead_code)] // This is for debugging
    pub fn dot(&self) -> impl std::fmt::Debug + '_ {
        Dot::new(&self.graph)
    }
}

#[derive(Default)]
pub struct Context {
    pub graph: Graph<Node, Edge>,
    schema_index: HashMap<Ref, NodeIndex>,
    pub operation_indices: Vec<NodeIndex>,
    errors: Vec<Error>,
    url: Option<Result<Url, Error>>,
}

impl TryFrom<Context> for ParseOutput {
    type Error = Vec<Error>;

    fn try_from(value: Context) -> Result<Self, Self::Error> {
        let Context { graph, errors, url, .. } = value;

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ParseOutput {
            graph,
            url: url.expect("parsing should always fill in url"),
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Ref(pub(self) String);

impl Ref {
    fn absolute(absolute: &str) -> Ref {
        Ref(absolute.to_string())
    }

    fn to_unresolved_error(&self) -> Error {
        Error::UnresolvedReference(self.to_string())
    }
}

impl std::fmt::Display for Ref {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn from_str<T: serde::de::DeserializeOwned>(data: &str, format: Format) -> Result<T, Vec<Error>> {
    Ok(match format {
        Format::Json => serde_json::from_str::<T>(data).map_err(|e| vec![Error::JsonParsingError(e.to_string())])?,
        Format::Yaml => serde_yaml::from_str::<T>(data).map_err(|e| vec![Error::YamlParsingError(e.to_string())])?,
    })
}
