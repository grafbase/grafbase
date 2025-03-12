use crate::wit;

/// HTTP headers.
pub struct HttpHeaders(wit::Headers);

/// HTTP headers for the gateway request.
pub struct GatewayHeaders(HttpHeaders);

impl From<wit::Headers> for GatewayHeaders {
    fn from(headers: wit::Headers) -> Self {
        Self(HttpHeaders(headers))
    }
}

impl std::ops::Deref for GatewayHeaders {
    type Target = HttpHeaders;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for GatewayHeaders {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// HTTP headers for the subgraph request.
pub struct SubgraphHeaders(HttpHeaders);

impl From<wit::Headers> for SubgraphHeaders {
    fn from(headers: wit::Headers) -> Self {
        Self(HttpHeaders(headers))
    }
}

impl std::ops::Deref for SubgraphHeaders {
    type Target = HttpHeaders;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SubgraphHeaders {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Imitates as much as possible the http::HeaderMap API
impl HttpHeaders {
    /// Get the value associated with the given name. If there are multiple values associated with
    /// the name, then the first one is returned. Use `get_all` to get all values associated with
    /// a given name. Returns None if there are no values associated with the name.
    pub fn get(&self, name: &str) -> Option<http::HeaderValue> {
        self.0
            .get(name)
            .into_iter()
            .next()
            .map(|value| value.try_into().unwrap())
    }

    /// Get all of the values corresponding to a name. If the name is not present,
    /// an empty list is returned. However, if the name is present but empty, this
    /// is represented by a list with one or more empty values present.
    pub fn get_all(&self, name: &str) -> impl Iterator<Item = http::HeaderValue> {
        self.0.get(name).into_iter().map(|value| value.try_into().unwrap())
    }

    /// Returns true if the map contains a value for the specified name.
    pub fn has(&self, name: &str) -> bool {
        self.0.has(name)
    }

    /// Set all of the values for a name. Clears any existing values for that
    /// name, if they have been set.
    pub fn set(&mut self, name: impl Into<http::HeaderName>, values: impl Iterator<Item: Into<http::HeaderValue>>) {
        let name = Into::<http::HeaderName>::into(name);
        let values = values
            .into_iter()
            .map(|value| Into::<http::HeaderValue>::into(value).as_bytes().to_vec())
            .collect::<Vec<_>>();
        self.0
            .set(name.as_str(), &values)
            .expect("We have a mut ref & validated name and values.");
    }

    /// Removes a name from the map, returning the value associated with the name.
    /// Returns None if the map does not contain the name. If there are multiple values associated with the name, then the first one is returned.
    pub fn remove(&mut self, name: &str) -> Option<http::HeaderValue> {
        self.0
            .get_and_delete(name)
            .map(|values| values.into_iter().next().map(|value| value.try_into().unwrap()))
            .expect("We have a mut ref & validated name and values.")
    }

    /// Append a value for a name. Does not change or delete any existing
    /// values for that name.
    pub fn append(&mut self, name: impl Into<http::HeaderName>, value: impl Into<http::HeaderValue>) {
        let name: http::HeaderName = name.into();
        let value: http::HeaderValue = value.into();
        self.0
            .append(name.as_str(), value.as_bytes())
            .expect("We have a mut ref & validated name and values.");
    }

    /// An iterator visiting all name-value pairs.
    /// The iteration order is arbitrary, but consistent across platforms for the same crate version. Each name will be yielded once per associated value. So, if a name has 3 associated values, it will be yielded 3 times.
    pub fn iter(&self) -> impl Iterator<Item = (http::HeaderName, http::HeaderValue)> {
        self.0
            .entries()
            .into_iter()
            .map(|(name, value)| (name.try_into().unwrap(), value.try_into().unwrap()))
    }
}

impl From<&GatewayHeaders> for http::HeaderMap {
    fn from(headers: &GatewayHeaders) -> Self {
        headers.iter().collect()
    }
}

impl From<&SubgraphHeaders> for http::HeaderMap {
    fn from(headers: &SubgraphHeaders) -> Self {
        headers.iter().collect()
    }
}

impl From<SubgraphHeaders> for http::HeaderMap {
    fn from(headers: SubgraphHeaders) -> Self {
        headers.iter().collect()
    }
}
