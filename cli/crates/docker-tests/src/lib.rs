#![allow(unused_crate_dependencies)]

use std::{
    future::Future,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::OnceLock,
    time::Duration,
};

use bollard::{
    container::{Config, CreateContainerOptions, LogOutput, LogsOptions, StopContainerOptions},
    secret::HostConfig,
    Docker,
};
use futures_util::TryStreamExt;
use tokio::{runtime::Runtime, time::sleep};

#[ctor::ctor]
fn setup_logging() {
    let filter = tracing_subscriber::filter::EnvFilter::builder()
        .parse(std::env::var("RUST_LOG").unwrap_or("grafbase_clickhouse_client=debug".to_string()))
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
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}

pub fn docker() -> &'static Docker {
    static DOCKER: OnceLock<Docker> = OnceLock::new();
    DOCKER.get_or_init(|| Docker::connect_with_local_defaults().unwrap())
}

pub fn with_grafbase<F>(data_dir: tempfile::TempDir, test: impl FnOnce(String) -> F)
where
    F: Future<Output = ()>,
{
    let Ok(docker_image) = std::env::var("GRAFBASE_DOCKER_IMAGE") else {
        unimplemented!("Please set GRAFBASE_DOCKER_IMAGE env var");
    };

    let (url, cleanup) = runtime().block_on(start_container(docker_image, data_dir));

    let result = catch_unwind(AssertUnwindSafe(|| runtime().block_on(test(url))));

    runtime().block_on(cleanup);

    if let Err(err) = result {
        std::panic::resume_unwind(err);
    }
}

#[allow(clippy::panic)]
async fn start_container(image_name: String, data_dir: tempfile::TempDir) -> (String, impl Future<Output = ()>) {
    let skip_cleanup = std::env::var("SKIP_CLEANUP").is_ok();
    let name = format!("grafbase_{}", ulid::Ulid::new()).to_lowercase();

    println!(
        "Using docker image: {image_name}\nContainer name: {name}\nData dir: {}",
        data_dir.path().display()
    );

    let docker = docker();
    docker
        .create_container(
            Some(CreateContainerOptions {
                name: name.clone(),
                platform: None,
            }),
            Config {
                image: Some(image_name),
                host_config: Some(HostConfig {
                    publish_all_ports: Some(true),
                    auto_remove: Some(true),
                    binds: Some(vec![format!("{}:/data", data_dir.path().display())]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    docker.start_container::<&str>(&name, None).await.unwrap();
    let inspection = docker.inspect_container(&name, None).await.unwrap();

    let binding = inspection
        .network_settings
        .and_then(|network| network.ports)
        .unwrap()
        .into_values()
        .find_map(|binding| binding)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();

    let addr = format!("{}:{}", binding.host_ip.unwrap(), binding.host_port.unwrap());
    let cleanup = async move {
        print_logs(&name).await;
        if skip_cleanup {
            println!("Skipping cleanup");
            std::mem::forget(data_dir);
        } else {
            let _ = docker.stop_container(&name, Some(StopContainerOptions { t: 1 })).await;
        }
    };

    // Leave some time for the gateway to fully start and port to be bound
    let wait_for_container = async {
        let start = std::time::Instant::now();
        println!("Waiting for grafbase to be available at {addr}...");
        // On MacOS port mapping takes forever (with colima at least), but on Linux it's sub
        // millisecond. CI is however not fast enough for the whole gateway to be fully started
        // before the test starts. So ensuring we always give the gateway some time.
        sleep(Duration::from_millis(100)).await;
        while tokio::net::TcpStream::connect(&addr).await.is_err() {
            sleep(Duration::from_millis(100)).await;
        }
        println!("Waited for {} ms", start.elapsed().as_millis());
    };

    // Leave some time for the gateway to fully start.
    tokio::select! {
        () = wait_for_container => {},
        () = sleep(Duration::from_secs(5)) => {
            cleanup.await;
            panic!("grafbase did not start in time");
        }
    }

    let url = format!("http://{addr}/graphql");
    (url, cleanup)
}

async fn print_logs(container_name: &str) {
    let result = crate::docker()
        .logs(
            container_name,
            Some(LogsOptions::<String> {
                stdout: true,
                stderr: true,
                ..Default::default()
            }),
        )
        .try_collect::<Vec<_>>()
        .await;
    if let Ok(logs) = result {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        for log in logs {
            match log {
                LogOutput::StdErr { message } => stderr.extend_from_slice(&message),
                LogOutput::StdOut { message } => stdout.extend_from_slice(&message),
                _ => {}
            }
        }
        if !stdout.is_empty() {
            println!("\n=== stdout ===\n{}", String::from_utf8_lossy(&stdout));
        }
        if !stderr.is_empty() {
            println!("\n=== stderr ===\n{}", String::from_utf8_lossy(&stderr));
        }
    }
}

pub async fn retry_for<T, F>(mut count: usize, delay: Duration, f: impl Fn() -> F) -> anyhow::Result<T>
where
    F: Future<Output = anyhow::Result<T>>,
{
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                count -= 1;
                if count == 0 {
                    return Err(e);
                }
                tokio::time::sleep(delay).await;
            }
        }
    }
}
