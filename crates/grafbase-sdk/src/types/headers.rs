pub(crate) use as_header_name::AsHeaderName;
pub(crate) use as_header_value::AsHeaderValue;

use crate::wit;

/// HTTP headers.
pub struct HttpHeaders(wit::Headers);

impl From<wit::Headers> for HttpHeaders {
    fn from(headers: wit::Headers) -> Self {
        Self(headers)
    }
}

impl From<HttpHeaders> for wit::Headers {
    fn from(headers: HttpHeaders) -> Self {
        headers.0
    }
}

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

impl From<SubgraphHeaders> for HttpHeaders {
    fn from(headers: SubgraphHeaders) -> Self {
        headers.0
    }
}

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

impl Default for HttpHeaders {
    fn default() -> Self {
        HttpHeaders::new()
    }
}

// Imitates as much as possible the http::HeaderMap API
impl HttpHeaders {
    /// Initialize an empty set of headers.
    pub fn new() -> HttpHeaders {
        HttpHeaders(wit::Headers::new())
    }

    /// Get the value associated with the given name. If there are multiple values associated with
    /// the name, then the first one is returned. Use `get_all` to get all values associated with
    /// a given name. Returns None if there are no values associated with the name.
    pub fn get(&self, name: impl AsHeaderName) -> Option<http::HeaderValue> {
        self.0
            .get(name.as_str())
            .into_iter()
            .next()
            .map(|value| value.try_into().unwrap())
    }

    /// Get all of the values corresponding to a name. If the name is not present,
    /// an empty list is returned. However, if the name is present but empty, this
    /// is represented by a list with one or more empty values present.
    pub fn get_all(&self, name: impl AsHeaderName) -> impl Iterator<Item = http::HeaderValue> {
        self.0
            .get(name.as_str())
            .into_iter()
            .map(|value| value.try_into().unwrap())
    }

    /// Returns true if the map contains a value for the specified name.
    pub fn has(&self, name: impl AsHeaderName) -> bool {
        self.0.has(name.as_str())
    }

    /// Set all of the values for a name. Clears any existing values for that
    /// name, if they have been set.
    pub fn set<V: AsHeaderValue>(&mut self, name: impl AsHeaderName, values: impl IntoIterator<Item = V>) {
        let values = values
            .into_iter()
            .map(|value| value.as_bytes().to_vec())
            .collect::<Vec<_>>();
        self.0
            .set(name.as_str(), &values)
            .expect("We have a mut ref & validated name and values.");
    }

    /// Removes a name from the map, returning the value associated with the name.
    /// Returns None if the map does not contain the name. If there are multiple values associated with the name, then the first one is returned.
    pub fn remove(&mut self, name: impl AsHeaderName) -> Option<http::HeaderValue> {
        self.0
            .get_and_delete(name.as_str())
            .map(|values| values.into_iter().next().map(|value| value.try_into().unwrap()))
            .expect("We have a mut ref & validated name and values.")
    }

    /// Append a value for a name. Does not change or delete any existing
    /// values for that name.
    pub fn append(&mut self, name: impl AsHeaderName, value: impl AsHeaderValue) {
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

/*
* Imitating the http::HeaderMap API
*
* ===== impl IntoHeaderName / AsHeaderName =====
*/

mod as_header_name {
    use http::HeaderName;

    /// A marker trait used to identify values that can be used as search keys
    /// to a `HttpHeaders`.
    pub trait AsHeaderName: Sealed {}

    // All methods are on this pub(super) trait, instead of `AsHeaderName`,
    // so that they aren't publicly exposed to the world.
    //
    // Being on the `AsHeaderName` trait would mean users could call
    // `"host".find(&map)`.
    //
    // Ultimately, this allows us to adjust the signatures of these methods
    // without breaking any external crate.
    pub trait Sealed {
        #[doc(hidden)]
        fn as_str(&self) -> &str;
    }

    // ==== impls ====

    impl Sealed for HeaderName {
        #[inline]
        fn as_str(&self) -> &str {
            HeaderName::as_str(self)
        }
    }

    impl AsHeaderName for HeaderName {}

    impl Sealed for &HeaderName {
        #[inline]
        fn as_str(&self) -> &str {
            HeaderName::as_str(self)
        }
    }

    impl AsHeaderName for &HeaderName {}

    impl Sealed for &str {
        #[inline]
        fn as_str(&self) -> &str {
            self
        }
    }

    impl AsHeaderName for &str {}

    impl Sealed for String {
        #[inline]
        fn as_str(&self) -> &str {
            String::as_str(self)
        }
    }

    impl AsHeaderName for String {}

    impl Sealed for &String {
        #[inline]
        fn as_str(&self) -> &str {
            String::as_str(self)
        }
    }

    impl AsHeaderName for &String {}
}

mod as_header_value {
    use http::HeaderValue;

    /// A marker trait used to identify values that can be used as search keys
    /// to a `HttpHeaders`.
    pub trait AsHeaderValue: Sealed {}

    // All methods are on this pub(super) trait, instead of `AsHeaderValue`,
    // so that they aren't publicly exposed to the world.
    //
    // Being on the `AsHeaderValue` trait would mean users could call
    // `"host".find(&map)`.
    //
    // Ultimately, this allows us to adjust the signatures of these methods
    // without breaking any external crate.
    pub trait Sealed: Sized {
        #[doc(hidden)]
        fn as_bytes(&self) -> &[u8];
        fn into_bytes(self) -> Vec<u8> {
            self.as_bytes().to_vec()
        }
    }

    // ==== impls ====

    impl Sealed for HeaderValue {
        #[inline]
        fn as_bytes(&self) -> &[u8] {
            HeaderValue::as_bytes(self)
        }
    }

    impl AsHeaderValue for HeaderValue {}

    impl Sealed for &HeaderValue {
        #[inline]
        fn as_bytes(&self) -> &[u8] {
            HeaderValue::as_bytes(self)
        }
    }

    impl AsHeaderValue for &HeaderValue {}

    impl Sealed for &[u8] {
        #[inline]
        fn as_bytes(&self) -> &[u8] {
            self
        }
    }

    impl AsHeaderValue for &[u8] {}

    impl Sealed for &str {
        #[inline]
        fn as_bytes(&self) -> &[u8] {
            str::as_bytes(self)
        }
    }

    impl AsHeaderValue for &str {}

    impl Sealed for String {
        #[inline]
        fn as_bytes(&self) -> &[u8] {
            String::as_bytes(self)
        }
        fn into_bytes(self) -> Vec<u8> {
            String::into_bytes(self)
        }
    }

    impl AsHeaderValue for String {}

    impl Sealed for &String {
        #[inline]
        fn as_bytes(&self) -> &[u8] {
            String::as_bytes(self)
        }
    }

    impl AsHeaderValue for &String {}

    impl Sealed for Vec<u8> {
        #[inline]
        fn as_bytes(&self) -> &[u8] {
            Vec::<u8>::as_ref(self)
        }
        fn into_bytes(self) -> Vec<u8> {
            self
        }
    }

    impl AsHeaderValue for Vec<u8> {}

    impl Sealed for &Vec<u8> {
        #[inline]
        fn as_bytes(&self) -> &[u8] {
            Vec::<u8>::as_ref(self)
        }
    }

    impl AsHeaderValue for &Vec<u8> {}
}
