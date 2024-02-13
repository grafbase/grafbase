# udf-wrapper

## Building

### With nix

```sh
$ nix build .#udf-wrapper
```

Whenever package-lock.json changes, you have to update the hash in `cli/nix/udf-wrapper.nix` for the nix build to keep working. You can get the hash by running `nix run nixpkgs#prefetch-npm-deps package-lock.json`.
