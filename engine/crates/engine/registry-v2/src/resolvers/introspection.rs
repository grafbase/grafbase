/// Some resolvers for implementing introspection
///
/// Currently most introspection is _not_ handled by these resolvers,
/// but instead by some legacy async_graphql code.  We want to get rid
/// of that sometime (ideally soon) though so expect we'll fill this out
/// sooner or later
#[serde_with::minify_variant_names(serialize = "minified", deserialize = "minified")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum IntrospectionResolver {
    FederationServiceField,
}
