#!/bin/bash
set -e

REGISTRY_URL="http://localhost:8081"
AUTH="-u admin:admin123"

echo "=== Publishing Demo Artifacts for All 8 Formats to TeaQL Registry ==="

# 1. Maven2
echo "1. Publishing Maven2 artifact..."
curl -s -f $AUTH -X PUT "$REGISTRY_URL/repository/maven-releases/com/example/teaql/teaql-sample-lib/1.0.0/teaql-sample-lib-1.0.0.pom" \
  -H "Content-Type: application/xml" \
  --data '<?xml version="1.0" encoding="UTF-8"?><project xmlns="http://maven.apache.org/POM/4.0.0"><modelVersion>4.0.0</modelVersion><groupId>com.example.teaql</groupId><artifactId>teaql-sample-lib</artifactId><version>1.0.0</version></project>'

curl -s -f $AUTH -X PUT "$REGISTRY_URL/repository/maven-releases/com/example/teaql/teaql-sample-lib/1.0.0/teaql-sample-lib-1.0.0.jar" \
  -H "Content-Type: application/java-archive" \
  --data 'Demo Maven Java JAR Bytecode'

# 2. NPM
echo "2. Publishing NPM package..."
curl -s -f $AUTH -X PUT "$REGISTRY_URL/repository/npm-hosted/npm/@teaql/sample-utils/-/sample-utils-1.0.0.tgz" \
  -H "Content-Type: application/gzip" \
  --data 'NPM Tarball Content'

# 3. PyPI
echo "3. Publishing PyPI package..."
curl -s -f $AUTH -X PUT "$REGISTRY_URL/repository/pypi-hosted/packages/teaql_client-1.0.0-py3-none-any.whl" \
  -H "Content-Type: application/x-wheel+zip" \
  --data 'Python Wheel Binary Data'

# 4. Docker
echo "4. Publishing Docker manifest..."
curl -s -f $AUTH -X PUT "$REGISTRY_URL/repository/docker-hosted/v2/library/teaql-demo-service/manifests/1.0.0" \
  -H "Content-Type: application/vnd.docker.distribution.manifest.v2+json" \
  --data '{"schemaVersion":2,"mediaType":"application/vnd.docker.distribution.manifest.v2+json","config":{"mediaType":"application/vnd.docker.container.image.v1+json","size":7023,"digest":"sha256:d826a7e0344d324b890887189196b27e69c10f607147b1981297e64a13e51f89"},"layers":[{"mediaType":"application/vnd.docker.image.rootfs.diff.tar.gzip","size":32654,"digest":"sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}]}'

# 5. Cargo (Rust)
echo "5. Publishing Cargo crate..."
curl -s -f $AUTH -X PUT "$REGISTRY_URL/repository/cargo-hosted/api/v1/crates/teaql-core-demo/0.1.0/download" \
  -H "Content-Type: application/gzip" \
  --data 'Rust Crate Archive Data'

# 6. Go Modules
echo "6. Publishing Go Module..."
curl -s -f $AUTH -X PUT "$REGISTRY_URL/repository/gomod-hosted/github.com/teaql/sample-go-lib/@v/v1.0.0.mod" \
  -H "Content-Type: text/plain" \
  --data 'module github.com/teaql/sample-go-lib

go 1.22'

curl -s -f $AUTH -X PUT "$REGISTRY_URL/repository/gomod-hosted/github.com/teaql/sample-go-lib/@v/v1.0.0.zip" \
  -H "Content-Type: application/zip" \
  --data 'Go Module Zip Archive'

# 7. NuGet (.NET)
echo "7. Publishing NuGet package..."
curl -s -f $AUTH -X PUT "$REGISTRY_URL/repository/nuget-hosted/v3/flatcontainer/teaql.sdk.dotnet/1.0.0/teaql.sdk.dotnet.1.0.0.nupkg" \
  -H "Content-Type: application/octet-stream" \
  --data 'NuGet Package Nupkg Data'

# 8. Raw Generic Binary
echo "8. Publishing Raw generic binary..."
curl -s -f $AUTH -X PUT "$REGISTRY_URL/repository/raw-hosted/dist/v1.0.0/teaql-cli-linux-amd64.tar.gz" \
  -H "Content-Type: application/gzip" \
  --data 'TeaQL CLI Release Binary Tarball'

echo "\n=== All 8 format artifacts published successfully! ==="
