{
  # Nix: https://nixos.org/download.html
  # How to activate flakes: https://nixos.wiki/wiki/Flakes
  # For seamless integration, consider using:
  # - direnv: https://github.com/direnv/direnv
  # - nix-direnv: https://github.com/nix-community/nix-direnv
  #
  # # .envrc
  # use flake
  # dotenv .env
  #
  description = "Grafbase CLI development environment";

  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs @ {
    flake-parts,
    nixpkgs,
    flake-utils,
    crane,
    rust-overlay,
    ...
  }: let
    inherit (nixpkgs.lib) optional concatStringsSep;
    systems = flake-utils.lib.system;
    flake = flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };

      aarch64DarwinExternalCargoCrates = concatStringsSep " " ["cargo-instruments@0.4.8" "cargo-about@0.6.1"];

      defaultShellConf = {
        nativeBuildInputs = with pkgs;
          [
            # Testing
            cargo-insta
            cargo-nextest
            cargo-component
            # Benchmark tool to send multiple requests
            hey

            # binary bloat inspector
            cargo-bloat

            # Versioning, automation and releasing
            cargo-make
            cargo-release
            nodePackages.npm
            nodePackages.semver
            sd

            # Node.js
            nodejs
            nodePackages.prettier
            bun # for wrappers

            # Native SSL
            openssl
            pkg-config
            cmake # for libz-ng-sys

            # Rust
            rustup

            # SQLx macros
            libiconv

            # Resolver tests
            pnpm # and cli-app
            yarn
          ]
          ++ optional (system == systems.aarch64-darwin) [
            cargo-binstall
            darwin.apple_sdk.frameworks.CoreFoundation
            darwin.apple_sdk.frameworks.CoreServices
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ]
          ++ optional (system != systems.aarch64-darwin) [
            gdb
            cargo-about # broken build at the moment on darwin
          ];

        shellHook = ''
          project_root="$(git rev-parse --show-toplevel 2>/dev/null || jj workspace root 2>/dev/null)"
          export POSTGRES_URL=postgresql://postgres:grafbase@localhost:5432
          export PGBOUNCER_URL=postgresql://postgres:grafbase@localhost:6432
          export CARGO_INSTALL_ROOT="$project_root/cli/.cargo";
          export PATH="$CARGO_INSTALL_ROOT/bin:$project_root/node_modules/.bin:$PATH";
          if [[ "${system}" == "aarch64-darwin" ]]; then
            cargo binstall --no-confirm --no-symlinks --quiet ${aarch64DarwinExternalCargoCrates}
          fi
        '';
      };
    in {
      devShells.default = pkgs.mkShell defaultShellConf;
    });
  in
    flake-parts.lib.mkFlake {inherit inputs;} {
      inherit flake;

      systems = flake-utils.lib.defaultSystems;

      perSystem = {
        config,
        system,
        ...
      }: {
        _module.args = {
          inherit crane;
          pkgs = import nixpkgs {
            inherit system;
            overlays = [(import rust-overlay)];
          };
        };

        imports = [./cli/nix/cli.nix ./nix/cli-app.nix ./gateway/nix/grafbase-gateway.nix];
      };
    };
}
