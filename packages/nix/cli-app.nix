{ pkgs, pnpm2nix, system, ... }:

let
  mkPnpmPackage = pnpm2nix.packages."${system}".mkPnpmPackage;

  extraIgnores = ''
    /crates
    /engine
    /cli
    /packages/nix
    *.nix
  '';
  src = pkgs.nix-gitignore.gitignoreSourcePure [ extraIgnores ".gitignore" ] ../../packages;
in
{
  packages.cli-app = mkPnpmPackage {
    inherit src;
    name = "cli-app";
    installInPlace = true;
    distDir = "../packages/cli-app/dist";
    script = "build-cli-app";
  };
}
