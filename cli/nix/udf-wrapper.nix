{ pkgs, ... }:

{
  packages.udf-wrapper = pkgs.buildNpmPackage {
    src = ../udf-wrapper;
    name = "udf-wrapper";
    npmDepsHash = "sha256-DParovYHR4sH0wFGZIrW5u3TAqIkqoFxAGtxvBCzAGo=";

    nativeBuildInputs = [ pkgs.bun ];

    installPhase = ''
      mkdir $out
      cp dist.js bun-multi-wrapper.ts $out/
    '';

    npmFlags = [ "--ignore-scripts" ];
  };
}
