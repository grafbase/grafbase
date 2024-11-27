use crate::{
    bindings::{
        component::grafbase::types::{Context, ErrorResponse, Headers},
        exports::component::grafbase::gateway_request,
    },
    init_logging, Component,
};

impl gateway_request::Guest for Component {}
