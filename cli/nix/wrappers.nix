{pkgs, ...}: {
  packages.wrappers = pkgs.buildNpmPackage {
    src = ../wrappers;
    name = "wrappers";
    npmDepsHash = "sha256-jZ1BC1A2rKASBbRqku3MSRasACUuebi1ekLm/aCkcDI=";

    nativeBuildInputs = [pkgs.bun];

    installPhase = ''
      mkdir $out
      cp dist.js bun-multi-wrapper.ts $out/
    '';

    npmFlags = ["--ignore-scripts"];
  };
}
