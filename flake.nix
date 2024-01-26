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
    pnpm2nix = {
      url = "github:nzbr/pnpm2nix-nzbr";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
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

  outputs = inputs@{
    flake-parts,
    nixpkgs,
    flake-utils,
    pnpm2nix,
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

      aarch64DarwinExternalCargoCrates = concatStringsSep " " ["cargo-instruments@0.4.8"];

      defaultShellConf = {
        nativeBuildInputs = with pkgs;
          [
            # Testing
            cargo-insta
            cargo-nextest

            # Versioning, automation and releasing
            cargo-about
            cargo-make
            cargo-release
            nodePackages.npm
            nodePackages.semver
            sd

            # Node.js
            nodejs
            nodePackages.prettier
            bun # for udf-wrapper

            # Native SSL
            openssl
            pkg-config

            # Rust
            rustup

            # SQLx macros
            libiconv

            # Resolver tests
            nodePackages.pnpm
            nodePackages.yarn
          ]
          ++ optional (system == systems.aarch64-darwin) [
            cargo-binstall
            darwin.apple_sdk.frameworks.CoreFoundation
            darwin.apple_sdk.frameworks.CoreServices
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];

        shellHook = ''
          project_root="$(git rev-parse --show-toplevel 2>/dev/null || jj workspace root 2>/dev/null)"
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
    flake-parts.lib.mkFlake { inherit inputs; } {
      inherit flake;

      systems = flake-utils.lib.defaultSystems;

      perSystem = { config, system, ... }: {
        _module.args = {
          inherit crane pnpm2nix;
          pkgs = import nixpkgs {
            inherit system;
            overlays = [(import rust-overlay)];
          };
        };

        imports = [ ./cli/nix/cli.nix ./packages/nix/cli-app.nix ];
      };
    };
}
