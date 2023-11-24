{ pkgs, crane, system, lib, ... }:

let
  toolchain = pkgs.rust-bin.stable.latest.minimal.override {
    targets = [ "wasm32-unknown-unknown" ];
  };
  craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;


  src = lib.cleanSourceWith {
    filter =
      (path: type:
        let
          isPest = builtins.isList (builtins.match ".*\\.pest$" path);
          isRust = craneLib.filterCargoSources path type;
        in
        isRust || isPest);
    src = lib.cleanSourceWith {
      filter = lib.cleanSourceFilter;
      src = (craneLib.path ../../.);
    };
  };

  commonArgs = {
    inherit src;
    pname = "engine-wasm";
    strictDeps = true;
    RUSTFLAGS = "-Aunused-crate-dependencies -Arust-2018-idioms";
    cargoExtraArgs = "-p engine-wasm --target=wasm32-unknown-unknown";
    doCheck = false;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  initialWasm = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
in
pkgs.writeShellScriptBin "engine-wasm" ''
  cargo install wasm-bindgen-cli@0.2.86 -q
  wasm-bindgen ${initialWasm}/lib/engine_wasm.wasm --target=nodejs --out-dir engine_wasm_stuff
''
