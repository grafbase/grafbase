use semver::Version;

const MINIMUM_GATEWAY_VERSION: Version = Version::new(0, 43, 0);

fn main() {
    let sdk_version = std::env::var("CARGO_PKG_VERSION").unwrap();

    let mut parts = sdk_version.split(|c: char| !c.is_ascii_digit());
    let major = parts.next().unwrap().parse::<u64>().unwrap();
    let minor = parts.next().unwrap().parse::<u64>().unwrap();
    let patch = parts.next().unwrap().parse::<u64>().unwrap();

    store_version("sdk_version_bytes", Version::new(major, minor, patch));
    store_version("minimum_gateway_version_bytes", MINIMUM_GATEWAY_VERSION);
}

fn store_version(file_name: &str, version: Version) {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let major = u16::try_from(version.major).unwrap().to_be_bytes();
    let minor = u16::try_from(version.minor).unwrap().to_be_bytes();
    let patch = u16::try_from(version.patch).unwrap().to_be_bytes();

    std::fs::write(
        std::path::Path::new(&out_dir).join(file_name),
        [major[0], major[1], minor[0], minor[1], patch[0], patch[1]],
    )
    .unwrap();
}
