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
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }: let
    inherit (nixpkgs.lib) optional;
    systems = flake-utils.lib.system;
  in
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };

      x86_64LinuxPkgs = import nixpkgs {
        inherit system;
        crossSystem = {
          config = "x86_64-unknown-linux-musl";
        };
      };
      x86_64LinuxBuildPkgs = x86_64LinuxPkgs.buildPackages;

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
            sd

            # DynamoDB local
            dynein

            # Node.js
            nodejs
            nodePackages.npm
            nodePackages.prettier
            nodePackages.semver

            # Native SSL
            openssl
            pkg-config

            # Rust
            cargo-binstall
            rustup

            # SQLx macros
            libiconv

            # Resolver tests
            nodePackages.pnpm
            nodePackages.yarn
          ]
          ++ optional (system == systems.aarch64-darwin) [
            darwin.apple_sdk.frameworks.CoreFoundation
            darwin.apple_sdk.frameworks.CoreServices
            darwin.apple_sdk.frameworks.Security
          ];

        shellHook = ''
          export CARGO_INSTALL_ROOT="$(git rev-parse --show-toplevel)/cli/.cargo";
          export PATH="$CARGO_INSTALL_ROOT/bin:$PATH";
            cargo binstall --no-confirm --no-symlinks --quiet cargo-sd@0.4.8
          if [[ "${system}" == "aarch64-darwin" ]]; then
            cargo binstall --no-confirm --no-symlinks --quiet cargo-instruments@0.4.8
          fi
        '';
      };
    in {
      devShells.default = pkgs.mkShell defaultShellConf;
      devShells.full = pkgs.mkShell (defaultShellConf
        // {
          buildInputs = with pkgs; [
            rustToolChain
            x86_64LinuxBuildPkgs.gcc
          ];

          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${x86_64LinuxBuildPkgs.gcc.out}/bin/x86_64-unknown-linux-gnu-gcc";
          CC_x86_64_unknown_linux_musl = "${x86_64LinuxBuildPkgs.gcc.out}/bin/x86_64-unknown-linux-gnu-gcc";
        });
      # Nightly Rust
      #
      # Clippy:
      #   nix develop .#nightly --command bash -c 'cd cli && cargo clippy --all-targets'
      #
      # Check Rust version:
      #   nix develop .#nightly --command bash -c 'echo "$PATH" | tr ":" "\n" | grep nightly'
      devShells.nightly = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          (rust-bin.selectLatestNightlyWith
            (toolchain:
              toolchain.minimal.override {
                extensions = ["clippy"];
              }))
        ];
      };
    });
}
