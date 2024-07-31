use crate::analyze::{AnalyzedSchema, Definition, Field};
use miette::{Diagnostic, SourceSpan};
use std::{ffi, fs, path::Path, rc::Rc};
use swc_common::{SourceFile, Span};
use swc_ecma_ast as ast;
use swc_ecma_parser as parser;
use thiserror::Error;

/// Sanity checks on resolvers to ensure that the right types are being used and the shape of
/// exports makes sense.
pub fn check_resolver(path: &Path, graphql_schema: &AnalyzedSchema<'_>) -> miette::Result<()> {
    let (src, module) = parse_file(path)?;
    let found_resolver_signature = find_resolver_signature(&module).map_err(|err| {
        err.with_source_code(miette::NamedSource::new(
            path.display().to_string(),
            src.as_str().to_owned(),
        ))
    })?;

    let Some((resolver_path_in_schema, resolver_field_span)) = found_resolver_signature else {
        return Ok(());
    };

    let Some((object_name, object_field)) = find_schema_field(resolver_path_in_schema, graphql_schema) else {
        return Ok(());
    };

    check_paths_match(path, object_field, object_name, resolver_field_span).map_err(|err| {
        err.with_source_code(miette::NamedSource::new(
            path.display().to_string(),
            src.as_str().to_owned(),
        ))
    })
}

fn check_paths_match(
    path: &Path,
    field: &Field<'_>,
    object_name: &str,
    resolver_field_span: Span,
) -> miette::Result<()> {
    #[derive(Debug, Diagnostic, Error)]
    #[error("Orphan resolver at {resolver_file_path}. The `{field}` field in your schema does not define a resolver.")]
    #[diagnostic(help(
        r#"Try declaring the resolver: `@resolver(name: "{suggested_resolver_name}")` in GraphQL or `.resolver("{suggested_resolver_name}")` in TypeScript."#
    ))]
    struct MissingResolverDirective {
        field: String,
        suggested_resolver_name: String,
        resolver_file_path: String,
        #[label("Inferred from this.")]
        span: SourceSpan,
    }

    #[derive(Debug, Diagnostic, Error)]
    #[error(r#"The resolver for `{field_name}` isn't at the right location. In your schema, the resolver for `{field_name}` is declared at `{resolver_name}`."#)]
    #[diagnostic(help(
        r#"You can move the file or change the resolver name in your schema to "{suggested_resolver_name}"."#
    ))]
    struct ResolverAtTheWrongPlace {
        resolver_name: String,
        field_name: String,
        suggested_resolver_name: String,
        #[label("Inferred from this.")]
        span: SourceSpan,
    }

    let Some(resolver_name) = &field.resolver_name else {
        return Err(MissingResolverDirective {
            field: format!("{object_name}.{}", field.name),
            suggested_resolver_name: suggested_resolver_name(path),
            span: swc_span_to_miette_span(resolver_field_span),
            resolver_file_path: path.display().to_string(),
        }
        .into());
    };

    // Now we have all the information about the resolver and what field it is supposed to resolve.
    let path_without_extension = path.with_extension("");
    let mut expected_path = resolver_name.split('/').collect::<Vec<&str>>();
    expected_path.reverse();
    let mut components = path_without_extension.components().rev().zip(expected_path.iter());
    if components.all(|(path, name)| path.as_os_str() == ffi::OsStr::new(name)) {
        return Ok(());
    }

    let field_name = &field.name;
    Err(ResolverAtTheWrongPlace {
        field_name: format!("{object_name}.{field_name}"),
        suggested_resolver_name: suggested_resolver_name(path),
        span: swc_span_to_miette_span(resolver_field_span),
        resolver_name: resolver_name.clone(),
    }
    .into())
}

/// Transform a `std::path::Path` into a suggested name for a resolver in `@resolver(name: "...")`.
fn suggested_resolver_name(path: &Path) -> String {
    let path_without_extension = path.with_extension("");
    let mut components = path_without_extension
        .components()
        .rev()
        .take_while(|component| component.as_os_str() != ffi::OsStr::new("resolvers"))
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>();
    components.reverse();
    components.join("/")
}

/// Takes a schema and a path of the form `User.fullName`, and returns the corresponding field, if
/// any.
///
/// Returns `(parent_object_name, field)`.
fn find_schema_field<'a>(
    resolver_path_in_schema: &'a str,
    graphql_schema: &'a AnalyzedSchema<'a>,
) -> Option<(&'a str, &'a Field<'a>)> {
    let mut split = resolver_path_in_schema.split('.');
    let type_name = split.next()?;
    let field_name = split.next()?;
    let Some(Definition::Object(object_id)) = graphql_schema.definition_names.get(type_name) else {
        return None;
    };
    graphql_schema
        .iter_object_fields(*object_id)
        .find(|field| field.name == field_name)
        .map(|field| (type_name, field))
}

/// Given a module whose default export has a type like `Resolver["User.fullname"]`, find and
/// return the `User.fullname` string and its location.
fn find_resolver_signature(module: &ast::Module) -> miette::Result<Option<(&str, Span)>> {
    let Some(resolver_ident) = find_default_export(module)? else {
        return Ok(None); // There is a default export, but we won't check it.
    };
    Ok(find_resolved_field_name(module, resolver_ident).map(|str_lit| (str_lit.value.as_ref(), str_lit.span)))
}

/// Given a module whose default export has name `ident`, find where it is declared, and if it has
/// a declaration that looks like `Resolver["User.fullname"]`, return the indexing string.
fn find_resolved_field_name<'a>(module: &'a ast::Module, ident: &ast::Ident) -> Option<&'a ast::Str> {
    let var_binding = module.body.iter().find_map(|item| {
        let var_decl = match item {
            ast::ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDecl(decl)) => decl.decl.as_var()?,
            ast::ModuleItem::Stmt(ast::Stmt::Decl(decl)) => decl.as_var()?,
            _ => return None,
        };

        var_decl
            .decls
            .first()?
            .name
            .as_ident()
            .filter(|binding| binding.id.sym == ident.sym)
    })?;

    var_binding
        .type_ann
        .as_ref()?
        .type_ann
        .as_ts_indexed_access_type()?
        .index_type
        .as_ts_lit_type()?
        .lit
        .as_str()
}

fn swc_span_to_miette_span(span: Span) -> SourceSpan {
    SourceSpan::new(
        // SWC spans seem to be 1 based where miette wants 0 based.
        miette::SourceOffset::from(span.lo.0 as usize - 1),
        (span.hi.0 - span.lo.0) as usize,
    )
}

fn find_default_export(module: &ast::Module) -> miette::Result<Option<&ast::Ident>> {
    #[derive(Debug, Diagnostic, Error)]
    #[diagnostic(help("Export your resolver as default: `export default resolver`",))]
    #[error("The module is missing a default export. Grafbase expects a resolver function as default export.")]
    struct MissingDefaultExport {
        #[label("Maybe this should be the default export?")]
        candidate: Option<SourceSpan>,
    }

    let mut has_default_export = false;
    let mut default_export_candidate = None;

    for item in &module.body {
        match item {
            ast::ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDecl(decl)) => {
                default_export_candidate = Some(swc_span_to_miette_span(decl.span));
            }
            ast::ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDefaultDecl(_)) => {
                has_default_export = true;
            }
            ast::ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDefaultExpr(expr)) => match expr.expr.as_ref() {
                ast::Expr::Ident(ident) => return Ok(Some(ident)),
                _ => {
                    has_default_export = true;
                }
            },
            _ => (),
        }
    }

    if has_default_export {
        Ok(None)
    } else {
        Err(MissingDefaultExport {
            candidate: default_export_candidate,
        }
        .into())
    }
}

fn parse_file(path: &Path) -> miette::Result<(Rc<String>, ast::Module)> {
    let mut recovered_errors = Vec::new(); // not used by us
    let source_file = path_to_swc_source_file(path)?;
    let src = source_file.src.clone();

    let module = parser::parse_file_as_module(
        &source_file,
        parser::Syntax::Typescript(parser::TsSyntax::default()),
        ast::EsVersion::EsNext,
        None,
        &mut recovered_errors,
    )
    .map_err(|_| {
        miette::miette!(
            severity = miette::Severity::Error,
            help =
                "The module must have a default export. See the docs: https://grafbase.com/docs/edge-gateway/resolvers",
            "The resolver at {} is not a valid TypeScript module.",
            path.display(),
        )
    })?;
    Ok((src, module))
}

#[derive(Debug, Error, miette::Diagnostic)]
#[error("Could not read the file.")]
struct CouldNotReadFile;

fn path_to_swc_source_file(path: &Path) -> Result<SourceFile, CouldNotReadFile> {
    use swc_common::source_map::SmallPos;
    let Ok(src) = fs::read_to_string(path) else {
        return Err(CouldNotReadFile);
    };
    let file_name = swc_common::FileName::Real(path.to_owned());
    Ok(SourceFile::new(
        Rc::new(file_name.clone()),
        false,
        Rc::new(file_name),
        src,
        swc_common::BytePos::from_u32(1),
    ))
}
