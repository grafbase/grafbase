{
  pkgs,
  mkPnpmPackage,
  ...
}: let
  extraIgnores = ''
    /crates
    /engine
    /cli
    /nix
    *.nix
  '';
  src = pkgs.nix-gitignore.gitignoreSource extraIgnores ../../.;
in
  mkPnpmPackage {
    inherit src;
    name = "cli-app";
    installInPlace = true;
    distDir = "packages/cli-app/dist";
    script = "build-cli-app";
  }
