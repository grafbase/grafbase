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
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }: let
    inherit
      (nixpkgs.lib)
      optionalAttrs
      ;
    systems = flake-utils.lib.system;
  in
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      rustToolChain = pkgs.rust-bin.fromRustupToolchainFile ./cli/rust-toolchain.toml;
    in {
      devShells.default = pkgs.mkShell (let
        common = {
          # Common dependencies
          nativeBuildInputs = with pkgs; [
            rustToolChain
            sccache

            # Miniflare
            nodejs

            # Formatting
            nodePackages.prettier
          ];

          RUSTC_WRAPPER = "${pkgs.sccache.out}/bin/sccache";

          shellHook = ''
            export CARGO_INSTALL_ROOT="$(git rev-parse --show-toplevel)/cli/.cargo"
            export PATH="$CARGO_INSTALL_ROOT/bin:$PATH"
          '';
        };
      in
        common
        # Linux-specific
        // optionalAttrs (system == systems.x86_64-linux) (let
          x86_64LinuxPkgs = import nixpkgs {
            inherit system;
            crossSystem = {
              config = "x86_64-unknown-linux-musl";
            };
          };
        in {
          nativeBuildInputs =
            common.nativeBuildInputs
            ++ (with pkgs; [
              pkg-config
            ]);
          # This is hack to avoid the redefinition of CC, CXX and so on to use aarch64.
          # There's probably a better way to do this.
          buildInputs = with pkgs; [
            x86_64LinuxPkgs.buildPackages.gcc
          ];

          CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${x86_64LinuxPkgs.buildPackages.gcc.out}/bin/x86_64-unknown-linux-gnu-gcc";
          CC_x86_64_unknown_linux_musl = "${x86_64LinuxPkgs.buildPackages.gcc.out}/bin/x86_64-unknown-linux-gnu-gcc";
        })
        # Darwin-specific
        // optionalAttrs (system == systems.aarch64-darwin) {
          nativeBuildInputs =
            common.nativeBuildInputs
            ++ (with pkgs; [
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.CoreFoundation
              darwin.apple_sdk.frameworks.CoreServices
            ]);
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
