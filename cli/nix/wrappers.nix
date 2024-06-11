{ pkgs, ... }: {
  packages.wrappers = pkgs.buildNpmPackage {
    src = ../wrappers;
    name = "wrappers";
    npmDepsHash = "sha256-ywx1TXl8WJW4YOSHJHkC7XktwQKnNakCLtjyTff293Q=";

    nativeBuildInputs = [ pkgs.bun ];

    installPhase = ''
      mkdir $out
      cp dist.js bun-multi-wrapper.ts parse-config.*ts $out/
    '';

    npmFlags = [ "--ignore-scripts" ];
  };
}
