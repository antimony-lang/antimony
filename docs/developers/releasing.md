# Release Workflow

1. Update version in `Cargo.toml`
2. Commit change with a meaningful message (`v0.1.1`)
3. Tag commit using `git tag -a <new release> -m "$(git shortlog <last release>..HEAD)"`
4. Push the tag using `git push --tags`
5. Publish package using `cargo publish`
