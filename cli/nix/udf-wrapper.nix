{ pkgs, ... }:

{
  packages.udf-wrapper = pkgs.buildNpmPackage {
    src = ../udf-wrapper;
    name = "udf-wrapper";
    npmDepsHash = "sha256-+74Y5QTvYVpX7Yv8SLp9+lQT5qk3ZNTXE0RksdEF3HI=";

    nativeBuildInputs = [ pkgs.bun ];

    installPhase = ''
      mkdir $out
      cp dist.js bun-multi-wrapper.ts $out/
    '';

    npmFlags = [ "--ignore-scripts" ];
  };
}
