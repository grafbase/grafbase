# wrappers

## Building

### With nix

```sh
$ nix build .#wrappers
```

Whenever package-lock.json changes, you have to update the hash in `cli/nix/wrappers.nix` for the nix build to keep working. You can get the hash by running `nix run nixpkgs#prefetch-npm-deps package-lock.json`.
