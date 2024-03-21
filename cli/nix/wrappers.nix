{pkgs, ...}: {
  packages.wrappers = pkgs.buildNpmPackage {
    src = ../wrappers;
    name = "wrappers";
    npmDepsHash = "sha256-DParovYHR4sH0wFGZIrW5u3TAqIkqoFxAGtxvBCzAGo=";

    nativeBuildInputs = [pkgs.bun];

    installPhase = ''
      mkdir $out
      cp dist.js bun-multi-wrapper.ts $out/
    '';

    npmFlags = ["--ignore-scripts"];
  };
}
