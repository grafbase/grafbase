{
  pkgs,
  mkPnpmPackage,
  ...
}: let
  extraIgnores = ''
    /crates
    /engine
    /cli
    /packages/nix
    *.nix
  '';
  src = pkgs.nix-gitignore.gitignoreSourcePure [extraIgnores ".gitignore"] ../../packages;
in
  mkPnpmPackage {
    inherit src;
    name = "cli-app";
    installInPlace = true;
    distDir = "../packages/cli-app/dist";
    script = "build-cli-app";
  }
