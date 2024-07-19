{pkgs, ...}: let
  src = pkgs.nix-gitignore.gitignoreSourcePure [".gitignore"] ../packages;
  pname = "cli-app";
  version = "1";

  inherit (pkgs) jq nodejs pnpm_9 stdenv;
in {
  packages.cli-app = stdenv.mkDerivation {
    inherit pname version src;

    pnpmDeps = pnpm_9.fetchDeps {
      inherit src pname version;
      hash = "sha256-v0PPuufzvNxSITOlFPb8j5IEySMpOLB65M/eRMvEVF8=";
    };

    buildInputs = [jq];

    nativeBuildInputs = [
      nodejs
      pnpm_9.configHook
    ];

    buildPhase = ''
      runHook preBuild

      jq ".packageManager = \"pnpm@${pnpm_9.version}\"" package.json > tmp.$$.json && mv tmp.$$.json package.json

      pnpm run build-cli-app
      cp -r ./cli-app/dist $out

      runHook postBuild
    '';
  };
}
