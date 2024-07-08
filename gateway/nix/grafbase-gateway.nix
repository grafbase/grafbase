{
  pkgs,
  crane,
  lib,
  ...
}: let
  rustToolchain = pkgs.rust-bin.stable.latest.default;
  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
  workspaceRoot = builtins.path {
    name = "grafbase-repo-src";
    path = ../../.;
  };

  # Aggressively prune the source tree for better caching.
  extraIgnores = ''
    *.nix
    *.md
    *.sh

    package.json
    assets.tar.gz

    /packages

    /flake.lock
    /renovate.json
    /scripts
    /packages/**/src

    node_modules/
    yarn.lock

    !/engine/crates/validation/README.md
    !/engine/crates/composition/README.md
    !/engine/crates/graphql-schema-diff/README.md
    !/packages/grafbase-sdk/package.json
  '';

  src = pkgs.nix-gitignore.gitignoreSource [extraIgnores] (lib.cleanSourceWith {
    filter = lib.cleanSourceFilter;
    src = workspaceRoot;
  });

  version = pkgs.runCommand "getVersion" {} ''
    ${pkgs.dasel}/bin/dasel \
      --file ${../../gateway/crates/gateway-binary/Cargo.toml} \
      --selector package.version\
      --write - | tr -d "\n" > $out
  '';
in {
  packages.grafbase-gateway = craneLib.buildPackage {
    inherit src;
    pname = "grafbase-gateway";
    version = builtins.readFile version;
    stdenv = pkgs.clangStdenv;

    cargoBuildFlags = "-p grafbase-gateway";
    cargoExtraArgs = "-p grafbase-gateway";

    RUSTFLAGS = builtins.concatStringsSep " " [
      "-Arust-2018-idioms -Aunused-crate-dependencies"
      "-C linker=clang -C link-arg=-fuse-ld=lld"
    ];

    doCheck = false;

    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.openssl.dev
      pkgs.llvmPackages.bintools # lld
    ];
  };
}
