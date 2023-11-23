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
      flake = false;
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
    rust-overlay,
    crane,
    ...
  }: let
    inherit (nixpkgs.lib) optional concatStringsSep;
    systems = flake-utils.lib.system;
    flake = flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        overlays = [(import rust-overlay)];
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

            # DynamoDB local
            dynein

            # Node.js
            nodejs
            nodePackages.prettier

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

      mkPnpmPackage = pnpm2nix.packages."${system}".mkPnpmPackage;
    in {
      devShells.default = pkgs.mkShell defaultShellConf;
      packages.cli-app = import ./packages/nix/cli-app.nix {inherit mkPnpmPackage pkgs;};
      packages.engine-wasm = import ./engine/nix/engine-wasm.nix {inherit pkgs system crane;};
    });
  in
    flake-parts.lib.mkFlake { inherit inputs; } {
      inherit flake;
      systems = flake-utils.lib.defaultSystems;
      perSystem = { config, ... }: {
      };
    };
}
