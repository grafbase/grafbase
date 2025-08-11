{pkgs, ...}: {
  packages = with pkgs; [
    rustup
    cargo-make
    cargo-nextest
    cargo-insta
    cargo-hakari
    git-cliff

    # TOML
    taplo

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
