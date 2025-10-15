# Versioning Strategy

This project uses a perpetual 0.x.y versioning scheme:

- **0.MAJOR.MINOR**
  - MAJOR (x): Breaking changes
  - MINOR (y): New features, backwards compatible

## Examples
- `0.1.0` → `0.2.0`: Breaking API changes
- `0.2.0` → `0.2.1`: Bug fixes or minor improvements
- `0.2.1` → `0.2.2`: Patches, documentation updates

## Docker Tags
- `latest`: Current development version
- `0.2.1`: Specific version
- `0.2`: Latest patch of 0.2.x series

## Release Process
1. Update version in `Cargo.toml`
2. Tag commit: `git tag v0.2.1`
3. Push tags: `git push --tags`
4. CI builds and pushes Docker images automatically

## Why 0.x.y Forever?
- Indicates experimental/research nature
- Allows breaking changes when needed
- Sets appropriate expectations for users