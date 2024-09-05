use std::path::PathBuf;

mod domain;
mod formatter;
mod generation;
mod loader;

const DOMAIN_DIR: &str = "domain";
const GENERATED_MODULE: &str = "generated";

fn main() -> anyhow::Result<()> {
    let code_gen_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let engine_v2_dir = code_gen_dir.parent().unwrap();

    let filter = tracing_subscriber::filter::EnvFilter::builder()
        .parse(std::env::var("RUST_LOG").unwrap_or("engine_v2_codegen=debug".to_string()))
        .unwrap();

    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(filter)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .without_time()
        .init();

    let formatter = formatter::Formatter::new()?;

    for entry in std::fs::read_dir(code_gen_dir.join(DOMAIN_DIR))? {
        let path = entry?.path();
        tracing::info!("Parsing {}", path.to_string_lossy());
        let domain = loader::load(path)?;
        let modules = generation::generate_modules(&formatter, &domain)?;

        let root_module_dir = engine_v2_dir.join(&domain.destination_path).join(GENERATED_MODULE);
        tracing::info!("Cleaning up existing code");
        let _ = std::fs::remove_dir_all(&root_module_dir);

        std::fs::create_dir_all(&root_module_dir)?;

        let mut root_submodules = Vec::new();
        for generation::GeneratedModule { module_path, contents } in modules {
            root_submodules.push(module_path.first().unwrap().clone());
            let mut path = root_module_dir.clone();
            for p in module_path {
                path.push(p);
            }
            path.set_extension("rs");

            std::fs::create_dir_all(path.parent().unwrap())?;
            std::fs::write(path, contents)?;
        }

        std::fs::write(
            root_module_dir.join("mod").with_extension("rs"),
            formatter.format(generation::generate_module_base_content(&domain, &root_submodules))?,
        )?;
    }

    Ok(())
}
