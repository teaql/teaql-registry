# Native client verification commands

Credentials are intentionally represented as `<redacted>`.

- Maven: `mvn deploy:deploy-file`, then `mvn dependency:get`.
- npm: `npm publish`, then `npm install` in a clean consumer directory.
- PyPI: `twine upload`, then `pip install` in a clean target directory.
- Cargo: `cargo publish --registry teaql`, assert sparse-index dependencies, then `cargo check` using the downloaded crate.
- Go modules: fixture publication followed by `go mod download` through `GOPROXY` (Go has no native publish operation).
- NuGet: `dotnet nuget push`, then `dotnet restore` from the v3 service index.
- Swift: `swift package-registry publish`, then registry-backed `swift package resolve` and `swift build`.
- Docker: `docker push`, local image removal, then `docker pull`.
- Raw: authenticated HTTP PUT and GET with SHA-256 comparison (there is no ecosystem package manager).
