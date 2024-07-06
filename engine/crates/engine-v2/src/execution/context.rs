use std::str::FromStr;

use ::runtime::hooks::Hooks;
use futures::future::BoxFuture;
use runtime::auth::AccessToken;
use schema::{HeaderRuleWalker, NameOrPatternRef, Schema};

use crate::{engine::RequestContext, Engine, Runtime};

use super::RequestHooks;

/// Context before starting to operation plan execution.
/// Background futures will be started in parallel to avoid delaying the plan.
pub(crate) struct PreExecutionContext<'ctx, R: Runtime> {
    pub(crate) engine: &'ctx Engine<R>,
    pub(crate) request_context: &'ctx RequestContext<<R::Hooks as Hooks>::Context>,
    pub(super) background_futures: crossbeam_queue::SegQueue<BoxFuture<'ctx, ()>>,
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    pub fn new(engine: &'ctx Engine<R>, request_context: &'ctx RequestContext<<R::Hooks as Hooks>::Context>) -> Self {
        Self {
            engine,
            request_context,
            background_futures: Default::default(),
        }
    }

    pub fn push_background_future(&mut self, future: BoxFuture<'ctx, ()>) {
        self.background_futures.push(future)
    }

    pub fn schema(&self) -> &'ctx Schema {
        &self.engine.schema
    }

    pub fn access_token(&self) -> &'ctx AccessToken {
        &self.request_context.access_token
    }

    pub fn headers(&self) -> &'ctx http::HeaderMap {
        &self.request_context.headers
    }

    pub fn hooks(&self) -> RequestHooks<'ctx, R::Hooks> {
        self.into()
    }
}

impl<'ctx, R: Runtime> std::ops::Deref for PreExecutionContext<'ctx, R> {
    type Target = Engine<R>;
    fn deref(&self) -> &'ctx Self::Target {
        self.engine
    }
}

/// Data available during the executor life during its build & execution phases.
pub(crate) struct ExecutionContext<'ctx, R: Runtime> {
    pub engine: &'ctx Engine<R>,
    pub request_context: &'ctx RequestContext<<R::Hooks as Hooks>::Context>,
}

impl<R: Runtime> Clone for ExecutionContext<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: Runtime> std::marker::Copy for ExecutionContext<'_, R> {}

impl<'ctx, R: Runtime> std::ops::Deref for ExecutionContext<'ctx, R> {
    type Target = Engine<R>;
    fn deref(&self) -> &'ctx Self::Target {
        self.engine
    }
}

impl<'ctx, R: Runtime> ExecutionContext<'ctx, R> {
    #[allow(unused)]
    pub fn access_token(&self) -> &'ctx AccessToken {
        &self.request_context.access_token
    }

    pub fn headers(&self) -> &'ctx http::HeaderMap {
        &self.request_context.headers
    }

    pub fn headers_with_rules(&self, rules: impl Iterator<Item = HeaderRuleWalker<'ctx>>) -> http::HeaderMap {
        let mut headers = http::HeaderMap::new();

        for header in rules {
            match header.rule() {
                schema::HeaderRuleRef::Forward { name, default, rename } => match name {
                    NameOrPatternRef::Pattern(regex) => {
                        let filtered = self.headers().iter().filter(|(key, _)| regex.is_match(key.as_str()));

                        for (name, value) in filtered {
                            match rename.and_then(|s| http::HeaderName::from_str(s).ok()) {
                                Some(rename) => {
                                    headers.insert(rename, value.clone());
                                }
                                None => {
                                    headers.insert(name.clone(), value.clone());
                                }
                            }
                        }
                    }
                    NameOrPatternRef::Name(name) => {
                        let header = self.headers().get(name);
                        let default = default.and_then(|d| http::HeaderValue::from_str(d).ok());

                        let name = match rename {
                            Some(rename) => rename,
                            None => name,
                        };

                        let Ok(name) = http::HeaderName::from_str(name) else {
                            continue;
                        };

                        match (header, default) {
                            (None, Some(default)) => {
                                headers.insert(name, default);
                            }
                            (Some(value), _) => {
                                headers.insert(name, value.clone());
                            }
                            _ => (),
                        };
                    }
                },
                schema::HeaderRuleRef::Insert { name, value } => {
                    let name = http::HeaderName::from_bytes(name.as_bytes()).ok();
                    let value = http::HeaderValue::from_str(value).ok();

                    if let Some((name, value)) = name.zip(value) {
                        headers.insert(name, value);
                    }
                }
                schema::HeaderRuleRef::Remove { name } => match name {
                    schema::NameOrPatternRef::Pattern(regex) => {
                        // https://github.com/hyperium/http/issues/632
                        let delete_list: Vec<_> = headers
                            .keys()
                            .filter(|key| regex.is_match(key.as_str()))
                            .map(Clone::clone)
                            .collect();

                        for key in delete_list {
                            headers.remove(key);
                        }
                    }
                    schema::NameOrPatternRef::Name(name) => {
                        headers.remove(name);
                    }
                },
            }
        }

        headers
    }

    #[allow(unused)]
    pub fn hooks(&self) -> RequestHooks<'ctx, R::Hooks> {
        self.into()
    }
}
