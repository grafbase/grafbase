use enumflags2::{bitflags, BitFlags};
use proc_macro::TokenStream;
use quote::quote;
use syn::ItemImpl;

const AUTHORIZE_EDGE_PRE_EXECUTION: &str = "authorize_edge_pre_execution";
const AUTHORIZE_NODE_PRE_EXECUTION: &str = "authorize_node_pre_execution";
const AUTHORIZE_PARENT_EDGE_POST_EXECUTION: &str = "authorize_parent_edge_post_execution";
const AUTHORIZE_EDGE_NODE_POST_EXECUTION: &str = "authorize_edge_node_post_execution";
const AUTHORIZE_EDGE_POST_EXECUTION: &str = "authorize_edge_post_execution";
const ON_GATEWAY_REQUEST: &str = "on_gateway_request";
const ON_SUBGRAPH_REQUEST: &str = "on_subgraph_request";
const ON_SUBGRAPH_RESPONSE: &str = "on_subgraph_response";
const ON_OPERATION_RESPONSE: &str = "on_operation_response";
const ON_HTTP_RESPONSE: &str = "on_http_response";

#[bitflags]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum HookImplementation {
    AuthorizeEdgePreExecution = 1 << 0,
    AuthorizeNodePreExecution = 1 << 1,
    AuthorizeParentEdgePostExecution = 1 << 2,
    AuthorizeEdgeNodePostExecution = 1 << 3,
    AuthorizeEdgePostExecution = 1 << 4,
    OnGatewayRequest = 1 << 5,
    OnSubgraphResponse = 1 << 6,
    OnOperationResponse = 1 << 7,
    OnHttpResponse = 1 << 8,
    OnSubgraphRequest = 1 << 9,
}

pub(super) fn expand(item: &ItemImpl) -> TokenStream {
    let mut implementations = BitFlags::empty();

    for item in &item.items {
        match item {
            syn::ImplItem::Fn(method) => {
                let name = &method.sig.ident;

                if name == AUTHORIZE_EDGE_PRE_EXECUTION {
                    implementations |= HookImplementation::AuthorizeEdgePreExecution;
                } else if name == AUTHORIZE_NODE_PRE_EXECUTION {
                    implementations |= HookImplementation::AuthorizeNodePreExecution;
                } else if name == AUTHORIZE_PARENT_EDGE_POST_EXECUTION {
                    implementations |= HookImplementation::AuthorizeParentEdgePostExecution
                } else if name == AUTHORIZE_EDGE_NODE_POST_EXECUTION {
                    implementations |= HookImplementation::AuthorizeEdgeNodePostExecution
                } else if name == AUTHORIZE_EDGE_POST_EXECUTION {
                    implementations |= HookImplementation::AuthorizeEdgePostExecution
                } else if name == ON_GATEWAY_REQUEST {
                    implementations |= HookImplementation::OnGatewayRequest
                } else if name == ON_SUBGRAPH_REQUEST {
                    implementations |= HookImplementation::OnSubgraphRequest
                } else if name == ON_SUBGRAPH_RESPONSE {
                    implementations |= HookImplementation::OnSubgraphResponse
                } else if name == ON_OPERATION_RESPONSE {
                    implementations |= HookImplementation::OnOperationResponse
                } else if name == ON_HTTP_RESPONSE {
                    implementations |= HookImplementation::OnHttpResponse
                }
            }
            _ => continue,
        }
    }

    let implementations = implementations.bits();

    let name = &item.self_ty;
    let (impl_generics, _, where_clause) = item.generics.split_for_impl();

    let token_stream = quote! {
        #item

        impl #impl_generics grafbase_hooks::HookImpls for #name #where_clause {
            fn hook_implementations(&self) -> u32 {
                #implementations
            }
        }
    };

    TokenStream::from(token_stream)
}
