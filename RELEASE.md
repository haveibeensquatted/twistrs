# Release

Twistrs following a semantic versioning scheme. For every release, the following steps must be undergone in order.

## Cycle

1. Compile latest `main` branch (`cargo b --all-features`);
2. Run entire test and doctest suite (`cargo t --all-features`);
3. Create new branch with release name (e.g. `0.1.3-beta`);
4. Open Pull Request with title, `Release $VERSION`;
5. Bump library version in [Cargo.toml](./twistrs/Cargo.toml);
6. Publish bumped version to [crates.io](https://crates.io/crates/twistrs);
7. Update library version in _all_ examples. If there are breaking changes, examples must be updated first;
8. Rerun entire test suite (`cargo t`);
9. Update CHANGELOG.md with list of changes;
10. Merge or request merge, of pull request to `main`. 
