use std::sync::Arc;
use wasmtime::component::{ComponentType, Lift, Lower};

use crate::InstanceState;

impl Host for InstanceState {}

#[derive(ComponentType, Lift, Lower)]
#[component(variant)]
#[derive(Clone)]
pub enum Token {
    #[component(name = "anonymous")]
    Anonymous,
    #[component(name = "bytes")]
    Bytes(Arc<[u8]>),
}

impl From<Token> for runtime::extension::Token {
    fn from(token: Token) -> Self {
        match token {
            Token::Anonymous => runtime::extension::Token::Anonymous,
            Token::Bytes(bytes) => runtime::extension::Token::Bytes(bytes),
        }
    }
}

impl From<runtime::extension::Token> for Token {
    fn from(token: runtime::extension::Token) -> Self {
        match token {
            runtime::extension::Token::Anonymous => Token::Anonymous,
            runtime::extension::Token::Bytes(bytes) => Token::Bytes(bytes),
        }
    }
}

// Typical Wasmtime bindgen! macro generated stuff
// It's really just unnecessary work to implement this when we can just call the function with the
// real type.
pub trait Host: Send + ::core::marker::Send {}
impl<_T: Host + ?Sized + Send> Host for &mut _T {}
pub fn add_to_linker<T, D>(
    _linker: &mut wasmtime::component::Linker<T>,
    _host_getter: fn(&mut T) -> D::Data<'_>,
) -> wasmtime::Result<()>
where
    D: wasmtime::component::HasData,
    for<'a> D::Data<'a>: Host,
    T: 'static + Send,
{
    Ok(())
}
