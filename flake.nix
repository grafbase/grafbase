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
  description = "Grafbase CLI development environment (linux)";

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
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      rustToolChain = pkgs.rust-bin.fromRustupToolchainFile ./cli/rust-toolchain.toml;
    in {
      devShells.default = pkgs.mkShell {
        # Extra inputs can be added here
        nativeBuildInputs = with pkgs; [
          # Rust
          rustToolChain
          openssl
          pkg-config
          nodejs
        ];

        shellHook = ''
          export CARGO_INSTALL_ROOT="$(git rev-parse --show-toplevel)/cli/.cargo"
          export PATH="$CARGO_INSTALL_ROOT/bin:$PATH"
        '';
      };
    });
}
