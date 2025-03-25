use grafbase_sdk::{
    AuthenticationExtension,
    types::{Configuration, Error, ErrorResponse, GatewayHeaders, Token},
};

#[derive(AuthenticationExtension)]
struct MyAuthentication;

impl AuthenticationExtension for MyAuthentication {
    fn new(_config: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    fn authenticate(&mut self, headers: &GatewayHeaders) -> Result<Token, ErrorResponse> {
        headers
            .get("current-user-id")
            .and_then(|value| value.to_str().ok()?.parse().ok())
            .map(|current_user_id| {
                // Here we create our custom token serialized with postcard, a very efficient
                // serialization library that can be used with serde. An even faster, but more
                // complex alternative would be rkyv used in the authorization extension.
                Token::from_bytes(postcard::to_stdvec(&common::Token { current_user_id }).unwrap())
            })
            .ok_or_else(|| {
                // If we can't find the current-user-id or it's invalid, the request will be denied
                // with this response and no further processing is done by the gateway.
                ErrorResponse::unauthorized().with_error(Error::new("Unauthenticated"))
            })
    }
}
