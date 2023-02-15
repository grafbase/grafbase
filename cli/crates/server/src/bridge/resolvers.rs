use std::collections::HashMap;
use std::path::Path;

use super::errors::ApiError;

#[derive(serde::Serialize)]
struct ResolverContext<'a> {
    env: &'a HashMap<String, String>,
}

#[derive(serde::Serialize)]
struct ResolverArgs<'a> {
    context: ResolverContext<'a>,
}

pub async fn invoke_resolver(
    resolvers_path: &Path,
    resolver_name: &str,
    environment_variables: &HashMap<String, String>,
) -> Result<serde_json::Value, ApiError> {
    trace!("resolver invocation\n\n{:#?}\n", resolver_name);

    let resolver_source_code = tokio::fs::read_to_string(resolvers_path.join(resolver_name).with_extension("js"))
        .await
        .map_err(|_| ApiError::ResolverDoesNotExist(resolver_name.to_owned()))?;

    let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
    let scope = &mut v8::HandleScope::new(isolate);
    let resource_name = v8::String::new(scope, resolver_name).unwrap();
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);
    let code = v8::String::new(scope, &resolver_source_code).unwrap();
    let source_map_url = v8::null(scope).into();
    let origin = v8::ScriptOrigin::new(
        scope,
        resource_name.into(),
        0,
        0,
        false,
        0,
        source_map_url,
        false,
        false,
        true,
    );

    let extract_error = |tc_scope: &mut v8::TryCatch<'_, v8::HandleScope<'_>>| {
        let error = tc_scope.exception().unwrap().to_rust_string_lossy(tc_scope);
        error!("v8 error: {error}");
        ApiError::ResolverInvalid(resolver_name.to_owned())
    };

    trace!("instantiating the module");

    let source = v8::script_compiler::Source::new(code, Some(&origin));
    let return_value = {
        let tc_scope = &mut v8::TryCatch::new(scope);

        let module = v8::script_compiler::compile_module(tc_scope, source).ok_or_else(|| extract_error(tc_scope))?;
        trace!("module compiled");

        module
            .instantiate_module(tc_scope, |_context, _string, _fixed_array, module| Some(module))
            .ok_or_else(|| extract_error(tc_scope))?;
        trace!("module instantiated");

        let _ = module.evaluate(tc_scope);
        let module_namespace: v8::Local<'_, v8::Object> = module.get_module_namespace().try_into().unwrap();
        let default_key = v8::String::new(tc_scope, "default").unwrap();
        let module_namespace = module_namespace.get(tc_scope, default_key.into()).unwrap();
        let default_function: v8::Local<'_, v8::Function> = module_namespace.try_into().map_err(|error| {
            error!("v8 error: {error}");
            ApiError::ResolverInvalid(resolver_name.to_owned())
        })?;

        let global = context.global(tc_scope).into();
        trace!("about to run the exported function");

        let arg = serde_v8::to_v8(
            tc_scope,
            ResolverArgs {
                context: ResolverContext {
                    env: environment_variables,
                },
            },
        )
        .expect("must be convertible to a v8 object");

        let return_value = default_function
            .call(tc_scope, global, &[arg])
            .ok_or_else(|| extract_error(tc_scope))?;
        serde_v8::from_v8(tc_scope, return_value)
    };

    return_value.map_err(|_| ApiError::ResolverInvalid(resolver_name.to_owned()))
}
