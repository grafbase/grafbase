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
  description = "Grafbase engine development environment";

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
    in {
      devShells.default =
        pkgs.mkShell
        {
          nativeBuildInputs = with pkgs;
            [
              # Rust
              cargo-make
              cmake
              openssl
              pkg-config
              rustup

              # for zstd
              libiconv

              # Used to generate CLI assets
              esbuild
              nodePackages.pnpm

              # Formatting
              nodePackages.prettier
            ]
            ++ optional (system == systems.aarch64-darwin) [
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.CoreFoundation
              darwin.apple_sdk.frameworks.CoreServices
              darwin.apple_sdk.frameworks.SystemConfiguration
            ];

          postShellHook = ''
            project_root="$(git rev-parse --show-toplevel 2>/dev/null || jj workspace root 2>/dev/null)"
            export CARGO_INSTALL_ROOT="$project_root/engine/.cargo"
            export PATH="$CARGO_INSTALL_ROOT/bin:$PATH"
          '';
        };
    });
}
