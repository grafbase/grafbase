{
  pkgs,
  crane,
  lib,
  config,
  ...
}: let
  rustToolchain = pkgs.rust-bin.stable.latest.default;
  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
  workspaceRoot = builtins.path {
    name = "grafbase-repo-src";
    path = ../../.;
  };

  # Agressively prune the source tree for better caching.
  extraIgnores = ''
    *.nix
    *.md
    *.sh

    package.json
    assets.tar.gz

    # We can't ignore /packages wholesale because we need to include
    # grafbase-sdk/package.json later.
    /packages/**/*.json
    /packages/**/*.js
    /packages/**/*.ts
    /packages/**/*.html

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
      --file ${../../Cargo.toml} \
      --selector workspace.package.version\
      --write - | tr -d "\n" > $out
  '';
in {
  imports = [
    ./wrappers.nix
  ];

  packages.cli = craneLib.buildPackage {
    inherit src;
    pname = "grafbase";
    version = builtins.readFile version;
    stdenv = pkgs.clangStdenv;

    cargoBuildFlags = "-p grafbase";

    RUSTFLAGS = builtins.concatStringsSep " " [
      "-Arust-2018-idioms -Aunused-crate-dependencies"
      "-C linker=clang -C link-arg=-fuse-ld=lld"
    ];

    GRAFBASE_CLI_PATHFINDER_BUNDLE_PATH = config.packages.cli-app;
    GRAFBASE_CLI_WRAPPERS_PATH = config.packages.wrappers;

    doCheck = false;

    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.openssl.dev
      pkgs.llvmPackages.bintools # lld
    ];
  };
}
