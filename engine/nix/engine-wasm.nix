{ pkgs, crane, system, ... }:

let
  toolchain = pkgs.rust-bin.stable.latest.minimal.override {
    targets = [ "wasm32-unknown-unknown" ];
  };
  craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

  src = craneLib.cleanCargoSource (craneLib.path ../../.);

  commonArgs = {
    inherit src;
    pname = "engine-wasm";
    strictDeps = true;
    RUSTFLAGS = "-Aunused-crate-dependencies";
    cargoExtraArgs = "-p engine-wasm --target=wasm32-unknown-unknown";
    doCheck = false;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; })
