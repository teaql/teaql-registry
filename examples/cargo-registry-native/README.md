# Native Cargo dependency gate

This disposable fixture distinguishes a real Cargo dependency graph from a registry that merely accepts and downloads `.crate` archives. The child crate depends on the base crate through the local `teaql` registry and actually derives a `serde` trait from crates.io. The script checks both dependencies in the post-publish sparse index; a fresh consumer must compile the graph and print `NATIVE_CARGO_TRANSITIVE_PASS value=42`.

Run `scripts/verify_native_cargo_client.sh` only against an isolated, localhost-mapped test registry with a fresh disposable database. It publishes two fixed test names/versions, so each run needs a fresh registry database. The script refuses a non-local index or an unset isolation acknowledgement. It does not stop the server or delete its database.
