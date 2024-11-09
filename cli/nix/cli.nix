{ pkgs
, crane
, lib
, config
, ...
}:
let
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

    /flake.lock
    /renovate.json
    /scripts

    !/engine/crates/validation/README.md
    !/engine/crates/composition/README.md
    !/engine/crates/graphql-schema-diff/README.md
  '';

  src = pkgs.nix-gitignore.gitignoreSource [ extraIgnores ] (lib.cleanSourceWith {
    filter = lib.cleanSourceFilter;
    src = workspaceRoot;
  });

  version = pkgs.runCommand "getVersion" { } ''
    ${pkgs.dasel}/bin/dasel \
      --file ${../../Cargo.toml} \
      --selector workspace.package.version\
      --write - | tr -d "\n" > $out
  '';
in
{
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

    doCheck = false;

    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.openssl.dev
      pkgs.llvmPackages.bintools # lld
    ];
  };
}
