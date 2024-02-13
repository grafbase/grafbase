{ pkgs, ... }:

{
  packages.udf-wrapper = pkgs.buildNpmPackage {
    src = ../udf-wrapper;
    name = "udf-wrapper";
    npmDepsHash = "sha256-xUNkgpuwy75kP1P7pVTguVDCQUa9nvArml2PyOMuBYM=";

    nativeBuildInputs = [ pkgs.bun ];

    installPhase = ''
      mkdir $out
      cp dist.js bun-multi-wrapper.ts $out/
    '';

    npmFlags = [ "--ignore-scripts" ];
  };
}
