# Release

Start with bumping the version:

```bash
> npm run bump-(patch|minor|major)
```

Edit the `changelog/version.md` with the changes added. Commit with the message `chore(sdk): Bump to version VERSION`, push and create a new pull request.

When the pull request is merged, run `npm run release`. Monitor the deployment on GitHub Actions.