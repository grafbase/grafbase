{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:
{
  packages = with pkgs; [
    rustup
    cargo-make
    cargo-nextest
    cargo-insta

    # Examples
    hurl
  ];

  # Federation audit tests
  languages.javascript = {
    enable = true;
    npm.enable = true;
  };
  languages.typescript.enable = true;
}
