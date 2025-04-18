# Release Workflow

1. Run prerelease checks: `sh contrib/prerelease_check.sh`
1. Update version in `Cargo.toml`
1. Add entry in `CHANGELOG.md`
1. Commit change with semantic version number (`v0.1.1`)
1. Tag commit using `git tag -a <new release> -m "$(git shortlog <last release>..HEAD)"`
1. Push the tag using `git push --tags`
1. Publish package using `cargo publish`
