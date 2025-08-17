mod abstract_types;
mod basic;
mod benchmarks;
mod cycle;
mod derive;
mod entities;
mod extension;
mod flatten;
mod inaccessible;
mod interface;
mod introspection;
mod lookup;
mod mutation;
mod provides;
mod requirements;
mod shared_root;
mod sibling_dependencies;
mod tea_shop;
mod typename;

use std::sync::OnceLock;

use itertools::Itertools;
use schema::Schema;
use tempfile::TempDir;
use tokio::runtime::Runtime;

#[ctor::ctor]
fn setup_logging() {
    let filter = tracing_subscriber::filter::EnvFilter::builder()
        .parse(std::env::var("RUST_LOG").unwrap_or("engine_query_planning=debug".to_string()))
        .unwrap();
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(filter)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .without_time()
        .init();
}

pub fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap()
    })
}

// Snapshot every step of the query solver
#[macro_export]
macro_rules! assert_solving_snapshots {
    ($name: expr, $schema: expr, $query: expr) => {
        let schema = $crate::tests::runtime().block_on($crate::tests::IntoSchema::from($schema).into_schema());
        let name = $name;
        let query = $query;
        let mut operation = ::operation::Operation::parse(&schema, None, query).unwrap();

        let ctx = ::operation::OperationContext {
            schema: &schema,
            operation: &operation,
        };

        let query_solution_space = $crate::Query::generate_solution_space(&schema, &operation).unwrap();
        insta::assert_snapshot!(
            format!("{name}-graph"),
            query_solution_space.to_dot_graph(ctx),
            &query_solution_space.to_pretty_dot_graph(ctx)
        );

        let mut solver = $crate::solve::Solver::initialize(&schema, &operation, query_solution_space).unwrap();
        insta::assert_snapshot!(
            format!("{name}-solver"),
            solver.to_dot_graph(false),
            &solver.to_pretty_dot_graph(false)
        );

        solver.execute().unwrap();
        insta::assert_snapshot!(
            format!("{name}-solved"),
            solver.to_dot_graph(false),
            &solver.to_pretty_dot_graph(false)
        );

        let solution = solver.into_solution();
        let crude_solved_query = solution.into_query(&schema, &mut operation).unwrap();
        let ctx = ::operation::OperationContext {
            schema: &schema,
            operation: &operation,
        };
        insta::assert_snapshot!(
            format!("{name}-partial-solution"),
            crude_solved_query.to_dot_graph(ctx),
            &crude_solved_query.to_pretty_dot_graph(ctx)
        );

        let solved_query = $crate::post_process::post_process(&schema, &mut operation, crude_solved_query);
        let ctx = ::operation::OperationContext {
            schema: &schema,
            operation: &operation,
        };
        insta::assert_snapshot!(
            format!("{name}-finalized-solution"),
            solved_query.to_dot_graph(ctx),
            &solved_query.to_pretty_dot_graph(ctx)
        );
    };
}

// Only includes the end from the steiner tree solution to avoid gigantic unreadable snapshots.
#[macro_export]
macro_rules! assert_solution_snapshots {
    ($name: expr, $schema: expr, $query: expr) => {
        let schema = $crate::tests::runtime().block_on($crate::tests::IntoSchema::from($schema).into_schema());
        let name = $name;
        let query = $query;
        let mut operation = ::operation::Operation::parse(&schema, None, query).unwrap();

        let query_solution_space = $crate::Query::generate_solution_space(&schema, &operation).unwrap();
        let mut solver = $crate::solve::Solver::initialize(&schema, &operation, query_solution_space).unwrap();

        solver.execute().unwrap();
        insta::assert_snapshot!(
            format!("{name}-solved"),
            solver.to_dot_graph(true),
            &solver.to_pretty_dot_graph(true)
        );

        let solution = solver.into_solution();
        let crude_solved_query = solution.into_query(&schema, &mut operation).unwrap();
        let ctx = ::operation::OperationContext {
            schema: &schema,
            operation: &operation,
        };
        insta::assert_snapshot!(
            format!("{name}-partial-solution"),
            crude_solved_query.to_dot_graph(ctx),
            &crude_solved_query.to_pretty_dot_graph(ctx)
        );

        let solved_query = $crate::post_process::post_process(&schema, &mut operation, crude_solved_query);
        let ctx = ::operation::OperationContext {
            schema: &schema,
            operation: &operation,
        };
        insta::assert_snapshot!(
            format!("{name}-finalized-solution"),
            solved_query.to_dot_graph(ctx),
            &solved_query.to_pretty_dot_graph(ctx)
        );
    };
}

#[allow(clippy::large_enum_variant)]
pub enum IntoSchema {
    Sdl(&'static str),
    Schema(Schema),
    WithExtensions(WithExtensions),
}

pub struct WithExtensions {
    pub sdl: &'static str,
    pub tmp_dir: TempDir,
    pub extensions: Vec<(url::Url, extension_catalog::Manifest)>,
}

impl WithExtensions {
    pub fn new(sdl: &'static str) -> Self {
        Self {
            sdl,
            tmp_dir: TempDir::new().expect("Failed to create temporary directory"),
            extensions: Vec::new(),
        }
    }

    pub fn resolver(mut self, url: &str, sdl: &str) -> Self {
        let url = url::Url::parse(url).expect("Invalid URL");
        let name = url.path_segments().unwrap().next_back().unwrap();

        let manifest = extension_catalog::Manifest {
            id: format!("{name}-1.0.0").parse().unwrap(),
            r#type: extension_catalog::Type::Resolver(Default::default()),
            sdk_version: "0.0.0".parse().unwrap(),
            minimum_gateway_version: "0.0.0".parse().unwrap(),
            description: String::new(),
            sdl: Some(sdl.into()),
            readme: None,
            homepage_url: None,
            repository_url: None,
            license: None,
            permissions: Default::default(),
            legacy_event_filter: Default::default(),
        };
        self.extensions.push((url, manifest));
        self
    }

    async fn into_schema(self) -> Schema {
        let Self {
            sdl,
            tmp_dir,
            extensions,
        } = self;

        let mut sdl = sdl.to_string();
        let mut catalog = extension_catalog::ExtensionCatalog::default();
        for (url, manifest) in extensions {
            let path = tmp_dir.path().join(manifest.name());
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(
                path.join("manifest.json"),
                serde_json::to_vec(&manifest.clone().into_versioned()).unwrap(),
            )
            .unwrap();
            let wasm_path = path.join("extension.wasm");
            std::fs::write(&wasm_path, b"wasm").unwrap();
            catalog.push(extension_catalog::Extension {
                config_key: manifest.name().to_string(),
                manifest,
                wasm_path,
            });
            sdl = sdl.replace(url.as_str(), url::Url::from_file_path(path).unwrap().as_str());
        }

        Schema::builder(&sdl).extensions(&catalog).build().await.unwrap()
    }
}

impl From<&'static str> for IntoSchema {
    fn from(sdl: &'static str) -> Self {
        IntoSchema::Sdl(sdl)
    }
}

impl From<Schema> for IntoSchema {
    fn from(schema: Schema) -> Self {
        IntoSchema::Schema(schema)
    }
}

impl From<WithExtensions> for IntoSchema {
    fn from(with_extensions: WithExtensions) -> Self {
        IntoSchema::WithExtensions(with_extensions)
    }
}

impl IntoSchema {
    pub async fn into_schema(self) -> Schema {
        match self {
            IntoSchema::Sdl(sdl) => Schema::from_sdl_or_panic(sdl).await,
            IntoSchema::Schema(schema) => schema,
            IntoSchema::WithExtensions(ext) => ext.into_schema().await,
        }
    }
}

#[allow(unused)]
fn strdiff(before: &str, after: &str) -> String {
    similar::TextDiff::from_lines(before, after)
        .iter_all_changes()
        .filter_map(|change| match change.tag() {
            similar::ChangeTag::Equal => None,
            similar::ChangeTag::Delete => Some(('-', change)),
            similar::ChangeTag::Insert => Some(('+', change)),
        })
        .format_with("", |(tag, change), f| f(&format_args!("{tag}{change}")))
        .to_string()
}
