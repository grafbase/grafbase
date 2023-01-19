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

      x86_64LinuxPkgs = import nixpkgs {
        inherit system;
        crossSystem = {
          config = "x86_64-unknown-linux-musl";
        };
      };

      rustToolChain = pkgs.rust-bin.fromRustupToolchainFile ./cli/rust-toolchain.toml;
    in {
      devShells.default = pkgs.mkShell {
        # Extra inputs can be added here
        nativeBuildInputs = with pkgs; [
          # Rust
          rustToolChain
          sccache
          openssl
          pkg-config
          nodejs

          # Formatting
          nodePackages.prettier
        ];

        # This is hack to avoid the redefinition of CC, CXX and so on to use aarch64.
        # There's probably a better way to do this.
        buildInputs = with pkgs; [
          x86_64LinuxPkgs.buildPackages.gcc
        ];

        RUSTC_WRAPPER = "${pkgs.sccache.out}/bin/sccache";
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = "${x86_64LinuxPkgs.buildPackages.gcc.out}/bin/x86_64-unknown-linux-gnu-gcc";
        CC_x86_64_unknown_linux_musl = "${x86_64LinuxPkgs.buildPackages.gcc.out}/bin/x86_64-unknown-linux-gnu-gcc";

        shellHook = ''
          export CARGO_INSTALL_ROOT="$(git rev-parse --show-toplevel)/cli/.cargo"
          export PATH="$CARGO_INSTALL_ROOT/bin:$PATH"
        '';
      };
    });
}
