{pkgs, ...}: {
  packages.wrappers = pkgs.buildNpmPackage {
    src = ../wrappers;
    name = "wrappers";
    npmDepsHash = "sha256-MUkBH6lAKUQLtm0TRX5zUThcQhi+wukUg6wLyOs+8Q8=";

    nativeBuildInputs = [pkgs.bun];

    installPhase = ''
      mkdir $out
      cp dist.js bun-multi-wrapper.ts parse-config.*ts $out/
    '';

    npmFlags = ["--ignore-scripts"];
  };
}
