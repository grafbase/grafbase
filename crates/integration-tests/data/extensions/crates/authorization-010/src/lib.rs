use grafbase_sdk::{
    AuthorizationExtension, Error, IntoQueryAuthorization, SubgraphHeaders, Token,
    types::{AuthorizationDecisions, Configuration, ErrorResponse, QueryElements, ResponseElements},
};

#[derive(AuthorizationExtension)]
struct CustomAuthorization;

#[derive(serde::Deserialize)]
struct DeniedIdsArgs {
    ids: Vec<u32>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
struct State {
    denied_ids: Vec<DeniedIds>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
struct DeniedIds {
    query_element_id: u32,
    denied_ids: Vec<u32>,
}

#[serde_with::serde_as]
#[derive(serde::Deserialize)]
struct Object<'a> {
    #[serde(borrow, flatten)]
    #[serde_as(as = "serde_with::Map<_, _>")]
    fields: Vec<(&'a str, serde_json::Value)>,
}

impl Object<'_> {
    fn id_as_u32(&self) -> u32 {
        println!("{:#?}", self.fields);
        self.fields.first().and_then(|(_, value)| value.as_u64()).unwrap() as u32
    }
}

impl AuthorizationExtension for CustomAuthorization {
    fn new(_: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    fn authorize_query(
        &mut self,
        headers: &mut SubgraphHeaders,
        token: Token,
        elements: QueryElements<'_>,
    ) -> Result<impl IntoQueryAuthorization, ErrorResponse> {
        let mut state = State { denied_ids: Vec::new() };
        let mut builder = AuthorizationDecisions::deny_some_builder();
        let error_id = builder.push_error(Error::new("Not authorized, query auth SDK010"));

        for (name, elements) in elements.iter_grouped_by_directive_name() {
            println!("{name}");
            match name {
                "deniedIds" => {
                    for element in elements {
                        let args: DeniedIdsArgs = element.arguments()?;
                        state.denied_ids.push(DeniedIds {
                            query_element_id: element.id().into(),
                            denied_ids: args.ids,
                        });
                    }
                }
                "deny" => {
                    for element in elements {
                        builder.deny_with_error_id(element, error_id);
                    }
                }
                _ => unreachable!(),
            }
        }

        headers.append(
            http::HeaderName::from_static("token"),
            http::HeaderValue::from_bytes(token.as_bytes().unwrap_or_default()).unwrap(),
        );

        let state = rkyv::api::high::to_bytes_in::<_, rkyv::rancor::Error>(&state, Vec::new()).unwrap();
        Ok((builder.build(), state))
    }

    fn authorize_response(
        &mut self,
        state: Vec<u8>,
        elements: ResponseElements<'_>,
    ) -> Result<AuthorizationDecisions, Error> {
        let state = rkyv::access::<ArchivedState, rkyv::rancor::Error>(&state).unwrap();
        let mut builder = AuthorizationDecisions::deny_some_builder();
        let error_id = builder.push_error(Error::new("Not authorized, response auth SDK010"));

        for element in elements {
            if let Some(denied) = state
                .denied_ids
                .iter()
                .find(|denied| denied.query_element_id == u32::from(element.query_element_id()))
            {
                for item in element.items() {
                    let object: Object<'_> = item.deserialize()?;
                    if denied.denied_ids.contains(&(object.id_as_u32().into())) {
                        builder.deny_with_error_id(item, error_id);
                    }
                }
            }
        }

        Ok(builder.build())
    }
}
