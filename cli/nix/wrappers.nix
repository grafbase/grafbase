{pkgs, ...}: {
  packages.wrappers = pkgs.buildNpmPackage {
    src = ../wrappers;
    name = "wrappers";
    npmDepsHash = "sha256-CFF1bNRbji5V6+VhFdYt6eptFZd1+QsOOj9tUu3xqdg=";

    nativeBuildInputs = [pkgs.bun];

    installPhase = ''
      mkdir $out
      cp dist.js bun-multi-wrapper.ts parse-config.*ts $out/
    '';

    npmFlags = ["--ignore-scripts"];
  };
}
