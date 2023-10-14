use serde_json::Value;

fn main() {
    let sdk_package = serde_json::from_slice::<Value>(
        &std::fs::read("../../../packages/grafbase-sdk/package.json")
            .expect("to be able to read the SDKs package.json"),
    )
    .expect("the SDK package.json to be JSON");

    let sdk_version = sdk_package["version"].as_str().expect("the version to be a string");

    println!("cargo:rustc-env=GRAFBASE_SDK_PACKAGE_VERSION=~{sdk_version}");
}
