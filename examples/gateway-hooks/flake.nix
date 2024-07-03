{
  description = "Julius Test project";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/master";
  inputs.devshell.url = "github:numtide/devshell";
  inputs.flake-parts.url = "github:hercules-ci/flake-parts";
  inputs.grafbase.url = "github:grafbase/grafbase/gateway-0.4.0";

  outputs = inputs @ {
    self,
    flake-parts,
    devshell,
    nixpkgs,
    grafbase,
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        devshell.flakeModule
      ];

      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "i686-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      perSystem = {
        pkgs,
        system,
        ...
      }: {
        devshells.default = {
          commands = [
            {
              package = pkgs.rustup;
              category = "development";
            }
            {
              package = pkgs.cargo-component;
              category = "development";
            }
            {
              package = pkgs.clang;
              category = "development";
            }
            {
              package = grafbase.packages.${system}.grafbase-gateway;
              category = "development";
              help = "The Grafbase self-hosted gateway";
            }
          ];
        };
      };
    };
}
