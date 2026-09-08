#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOSITORY_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
REGISTRY_URL="${REGISTRY_URL:-http://127.0.0.1:8081}"
CREDENTIALS_DIR="${CREDENTIALS_DIR:-/var/lib/teaql-registry/credentials}"
RUN_ID="${NATIVE_RUN_ID:-$(date -u +%Y%m%d%H%M%S)}"
RESULTS_DIR="${NATIVE_RESULTS_DIR:-${REPOSITORY_ROOT}/evidence/native-clients/${RUN_ID}}"
WORK_DIR="$(mktemp -d /tmp/teaql-registry-native.XXXXXX)"
TOKEN_ID=""
PAT=""
DOCKER_IMAGE=""
SOURCE_GIT_COMMIT="$(git -C "${REPOSITORY_ROOT}" rev-parse HEAD)"
SOURCE_GIT_DIRTY="$(if [[ -z "$(git -C "${REPOSITORY_ROOT}" status --porcelain)" ]]; then echo false; else echo true; fi)"

if [[ -f "${CREDENTIALS_DIR}/global-admin.txt" ]]; then
  ADMIN_PASSWORD_VALUE="$(sed -n 's/^admin \/ //p' "${CREDENTIALS_DIR}/global-admin.txt" | head -n 1)"
else
  : "${ADMIN_PASSWORD:?Set ADMIN_PASSWORD or provide ${CREDENTIALS_DIR}/global-admin.txt}"
  ADMIN_PASSWORD_VALUE="${ADMIN_PASSWORD}"
fi

required_commands=(base64 cargo curl docker dotnet go jar jq mvn npm perl python3 sha256sum unzip zip)
for command_name in "${required_commands[@]}"; do
  if ! command -v "${command_name}" >/dev/null 2>&1; then
    echo "error: required native client or build tool is missing: ${command_name}" >&2
    exit 2
  fi
done

cleanup() {
  if [[ -n "${DOCKER_IMAGE}" ]]; then
    docker image rm "${DOCKER_IMAGE}" >/dev/null 2>&1 || true
  fi
  if [[ -n "${TOKEN_ID}" ]]; then
    curl -fsS --user "admin:${ADMIN_PASSWORD_VALUE}" -X DELETE \
      "${REGISTRY_URL}/service/rest/v1/tokens/${TOKEN_ID}" >/dev/null 2>&1 || true
  fi
  chmod -R u+w "${WORK_DIR}" 2>/dev/null || true
  rm -rf "${WORK_DIR}"
}
trap cleanup EXIT INT TERM

mkdir -p "${RESULTS_DIR}"
SUMMARY="${RESULTS_DIR}/summary.tsv"
printf 'format\tpublish\tconsume\n' >"${SUMMARY}"

sanitize_log() {
  local source="$1"
  local destination="$2"
  TEAQL_SECRET_PASSWORD="${ADMIN_PASSWORD_VALUE}" TEAQL_SECRET_PAT="${PAT}" perl -pe '
    s/\Q$ENV{TEAQL_SECRET_PASSWORD}\E/<redacted>/g if length $ENV{TEAQL_SECRET_PASSWORD};
    s/\Q$ENV{TEAQL_SECRET_PAT}\E/<redacted>/g if length $ENV{TEAQL_SECRET_PAT};
  ' "${source}" >"${destination}"
  rm -f "${source}"
}

run_step() {
  local label="$1"
  shift
  local raw_log="${WORK_DIR}/${label}.raw.log"
  local status=0
  "$@" >"${raw_log}" 2>&1 || status=$?
  sanitize_log "${raw_log}" "${RESULTS_DIR}/${label}.txt"
  if [[ "${status}" -eq 0 ]]; then
    return 0
  fi
  echo "error: ${label} failed; see ${RESULTS_DIR}/${label}.txt" >&2
  return "${status}"
}

curl -fsS --user "admin:${ADMIN_PASSWORD_VALUE}" "${REGISTRY_URL}/help" >/dev/null
TOKEN_RESPONSE="$(curl -fsS --user "admin:${ADMIN_PASSWORD_VALUE}" \
  -H 'Content-Type: application/json' \
  -d '{"description":"native client conformance run","scopes":["repository:read","repository:write"],"expires_in_days":1}' \
  "${REGISTRY_URL}/service/rest/v1/tokens")"
PAT="$(jq -er '.token' <<<"${TOKEN_RESPONSE}")"
TOKEN_ID="$(jq -er '.pat.id' <<<"${TOKEN_RESPONSE}")"

{
  echo "run_id=${RUN_ID}"
  echo "recorded_at_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "git_commit=${SOURCE_GIT_COMMIT}"
  echo "git_dirty=${SOURCE_GIT_DIRTY}"
  echo "registry_url=${REGISTRY_URL}"
  echo "maven=$(mvn --version | head -n 1)"
  echo "npm=$(npm --version)"
  echo "python=$(python3 --version 2>&1)"
  echo "cargo=$(cargo --version)"
  echo "go=$(go version)"
  echo "dotnet=$(dotnet --version)"
  echo "docker=$(docker --version)"
} >"${RESULTS_DIR}/environment.txt"

cat >"${RESULTS_DIR}/commands.md" <<'EOF'
# Native client verification commands

Credentials are intentionally represented as `<redacted>`.

- Maven: `mvn deploy:deploy-file`, then `mvn dependency:get`.
- npm: `npm publish`, then `npm install` in a clean consumer directory.
- PyPI: `twine upload`, then `pip install` in a clean target directory.
- Cargo: `cargo publish --registry teaql`, then `cargo fetch` from the sparse registry.
- Go modules: fixture publication followed by `go mod download` through `GOPROXY` (Go has no native publish operation).
- NuGet: `dotnet nuget push`, then `dotnet restore` from the v3 service index.
- Docker: `docker push`, local image removal, then `docker pull`.
- Raw: authenticated HTTP PUT and GET with SHA-256 comparison (there is no ecosystem package manager).
EOF

# Maven
MAVEN_ARTIFACT="native-probe-${RUN_ID}"
mkdir -p "${WORK_DIR}/maven/content" "${WORK_DIR}/maven/local"
printf 'native maven probe %s\n' "${RUN_ID}" >"${WORK_DIR}/maven/content/probe.txt"
jar --create --file "${WORK_DIR}/maven/${MAVEN_ARTIFACT}.jar" -C "${WORK_DIR}/maven/content" .
cat >"${WORK_DIR}/maven/settings.xml" <<EOF
<settings><servers><server><id>teaql-native</id><username>admin</username><password>${PAT}</password></server></servers></settings>
EOF
run_step maven-publish mvn -B -s "${WORK_DIR}/maven/settings.xml" deploy:deploy-file \
  -DrepositoryId=teaql-native -Durl="${REGISTRY_URL}/repository/maven-releases/" \
  -DgroupId=io.teaql.probe -DartifactId="${MAVEN_ARTIFACT}" -Dversion=0.0.1 \
  -Dpackaging=jar -DgeneratePom=true -Dfile="${WORK_DIR}/maven/${MAVEN_ARTIFACT}.jar"
run_step maven-consume mvn -B -s "${WORK_DIR}/maven/settings.xml" \
  -Dmaven.repo.local="${WORK_DIR}/maven/local" dependency:get \
  -Dartifact="io.teaql.probe:${MAVEN_ARTIFACT}:0.0.1" -Dtransitive=false \
  -DremoteRepositories="teaql-native::default::${REGISTRY_URL}/repository/maven-releases/"
test -f "${WORK_DIR}/maven/local/io/teaql/probe/${MAVEN_ARTIFACT}/0.0.1/${MAVEN_ARTIFACT}-0.0.1.jar"
printf 'Maven\tPASS\tPASS\n' >>"${SUMMARY}"

# npm
NPM_PACKAGE="teaql-native-probe-${RUN_ID}"
mkdir -p "${WORK_DIR}/npm/package" "${WORK_DIR}/npm/consumer"
cat >"${WORK_DIR}/npm/package/package.json" <<EOF
{"name":"${NPM_PACKAGE}","version":"0.0.1","main":"index.js"}
EOF
printf 'module.exports = "%s";\n' "${RUN_ID}" >"${WORK_DIR}/npm/package/index.js"
NPM_AUTH="$(printf 'admin:%s' "${PAT}" | base64 -w 0)"
NPM_REGISTRY_PATH="${REGISTRY_URL#*://}/repository/npm-hosted/npm/"
cat >"${WORK_DIR}/npm/npmrc" <<EOF
registry=${REGISTRY_URL}/repository/npm-hosted/npm/
//${NPM_REGISTRY_PATH}:_auth=${NPM_AUTH}
always-auth=true
EOF
run_step npm-publish env NPM_CONFIG_USERCONFIG="${WORK_DIR}/npm/npmrc" npm publish "${WORK_DIR}/npm/package" --registry "${REGISTRY_URL}/repository/npm-hosted/npm/"
run_step npm-consume env NPM_CONFIG_USERCONFIG="${WORK_DIR}/npm/npmrc" npm install --prefix "${WORK_DIR}/npm/consumer" --ignore-scripts "${NPM_PACKAGE}@0.0.1"
test -f "${WORK_DIR}/npm/consumer/node_modules/${NPM_PACKAGE}/index.js"
printf 'npm\tPASS\tPASS\n' >>"${SUMMARY}"

# PyPI
PYPI_PACKAGE="teaql_native_probe_${RUN_ID}"
mkdir -p "${WORK_DIR}/python/src/${PYPI_PACKAGE}"
cat >"${WORK_DIR}/python/pyproject.toml" <<EOF
[build-system]
requires = ["setuptools>=61"]
build-backend = "setuptools.build_meta"
[project]
name = "${PYPI_PACKAGE}"
version = "0.0.1"
EOF
printf '__version__ = "0.0.1"\n' >"${WORK_DIR}/python/src/${PYPI_PACKAGE}/__init__.py"
run_step pypi-build python3 -m build --wheel --no-isolation "${WORK_DIR}/python"
if command -v twine >/dev/null 2>&1; then
  TWINE=(twine)
else
  python3 -m venv "${WORK_DIR}/twine-venv"
  run_step pypi-twine-install "${WORK_DIR}/twine-venv/bin/pip" install --disable-pip-version-check twine
  TWINE=("${WORK_DIR}/twine-venv/bin/twine")
fi
run_step pypi-publish "${TWINE[@]}" upload --non-interactive \
  --repository-url "${REGISTRY_URL}/repository/pypi-hosted/pypi/upload" \
  --username admin --password "${PAT}" "${WORK_DIR}/python/dist/"*.whl
cat >"${WORK_DIR}/python/pip.conf" <<EOF
[global]
index-url = ${REGISTRY_URL/\/\//\/\/admin:${PAT}@}/repository/pypi-hosted/simple/
disable-pip-version-check = true
EOF
run_step pypi-consume env PIP_CONFIG_FILE="${WORK_DIR}/python/pip.conf" python3 -m pip install \
  --no-deps --target "${WORK_DIR}/python/consumer" "${PYPI_PACKAGE}==0.0.1"
test -f "${WORK_DIR}/python/consumer/${PYPI_PACKAGE}/__init__.py"
printf 'PyPI\tPASS\tPASS\n' >>"${SUMMARY}"

# Cargo
CARGO_PACKAGE="teaql-native-probe-${RUN_ID}"
mkdir -p "${WORK_DIR}/cargo/package/src" "${WORK_DIR}/cargo/consumer/src"
cat >"${WORK_DIR}/cargo/package/Cargo.toml" <<EOF
[package]
name = "${CARGO_PACKAGE}"
version = "0.0.1"
edition = "2021"
license = "MIT"
description = "TeaQL Registry native client probe"
EOF
printf 'pub fn probe() -> &\x27static str { "%s" }\n' "${RUN_ID}" >"${WORK_DIR}/cargo/package/src/lib.rs"
cat >"${WORK_DIR}/cargo/config.toml" <<EOF
[registries.teaql]
index = "sparse+${REGISTRY_URL}/repository/cargo-hosted/cargo/index/"
credential-provider = "cargo:token"
EOF
mkdir -p "${WORK_DIR}/cargo/home"
cp "${WORK_DIR}/cargo/config.toml" "${WORK_DIR}/cargo/home/config.toml"
run_step cargo-publish env CARGO_HOME="${WORK_DIR}/cargo/home" \
  CARGO_REGISTRIES_TEAQL_INDEX="sparse+${REGISTRY_URL}/repository/cargo-hosted/cargo/index/" \
  CARGO_REGISTRIES_TEAQL_TOKEN="Bearer ${PAT}" cargo publish \
  --manifest-path "${WORK_DIR}/cargo/package/Cargo.toml" --registry teaql --allow-dirty
cat >"${WORK_DIR}/cargo/consumer/Cargo.toml" <<EOF
[package]
name = "teaql-native-consumer-${RUN_ID}"
version = "0.0.1"
edition = "2021"
[dependencies]
${CARGO_PACKAGE} = { version = "=0.0.1", registry = "teaql" }
EOF
printf 'fn main() {}\n' >"${WORK_DIR}/cargo/consumer/src/main.rs"
run_step cargo-consume env CARGO_HOME="${WORK_DIR}/cargo/home" \
  CARGO_REGISTRIES_TEAQL_INDEX="sparse+${REGISTRY_URL}/repository/cargo-hosted/cargo/index/" \
  CARGO_REGISTRIES_TEAQL_TOKEN="Bearer ${PAT}" cargo fetch \
  --manifest-path "${WORK_DIR}/cargo/consumer/Cargo.toml"
printf 'Cargo\tPASS\tPASS\n' >>"${SUMMARY}"

# Go modules (native consumption; GOPROXY defines no publication operation)
GO_MODULE="example.com/teaql/nativeprobe${RUN_ID}"
GO_VERSION="v0.0.1"
GO_FIXTURE="${WORK_DIR}/go/fixture/${GO_MODULE}@${GO_VERSION}"
mkdir -p "${GO_FIXTURE}" "${WORK_DIR}/go/consumer"
printf 'module %s\n\ngo 1.18\n' "${GO_MODULE}" >"${GO_FIXTURE}/go.mod"
printf 'package nativeprobe\nconst Version = "%s"\n' "${RUN_ID}" >"${GO_FIXTURE}/probe.go"
(cd "${WORK_DIR}/go/fixture" && zip -qr "${WORK_DIR}/go/module.zip" "${GO_MODULE}@${GO_VERSION}")
curl -fsS --user "admin:${PAT}" -X PUT --data-binary "@${GO_FIXTURE}/go.mod" \
  "${REGISTRY_URL}/repository/gomod-hosted/gomod/${GO_MODULE}/@v/${GO_VERSION}.mod" >/dev/null
curl -fsS --user "admin:${PAT}" -X PUT --data-binary "@${WORK_DIR}/go/module.zip" \
  "${REGISTRY_URL}/repository/gomod-hosted/gomod/${GO_MODULE}/@v/${GO_VERSION}.zip" >/dev/null
printf '{"Version":"%s","Time":"%s"}\n' "${GO_VERSION}" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >"${WORK_DIR}/go/info.json"
curl -fsS --user "admin:${PAT}" -X PUT --data-binary "@${WORK_DIR}/go/info.json" \
  "${REGISTRY_URL}/repository/gomod-hosted/gomod/${GO_MODULE}/@v/${GO_VERSION}.info" >/dev/null
cat >"${WORK_DIR}/go/consumer/go.mod" <<EOF
module example.com/teaql/consumer${RUN_ID}
go 1.18
require ${GO_MODULE} ${GO_VERSION}
EOF
if [[ "${REGISTRY_URL}" == http://* ]]; then
  GO_PROXY_URL="${REGISTRY_URL}/repository/gomod-hosted/gomod"
else
  GO_PROXY_URL="${REGISTRY_URL/\/\//\/\/admin:${PAT}@}/repository/gomod-hosted/gomod"
fi
run_step go-consume env GOPROXY="${GO_PROXY_URL}" GOSUMDB=off \
  GOMODCACHE="${WORK_DIR}/go/modcache" go mod download -x "${GO_MODULE}@${GO_VERSION}"
printf 'GoMod\tN/A\tPASS\n' >>"${SUMMARY}"

# NuGet
NUGET_ID="TeaQL.Native.Probe.${RUN_ID}"
mkdir -p "${WORK_DIR}/nuget/package" "${WORK_DIR}/nuget/consumer"
run_step nuget-create dotnet new classlib --no-restore --force -o "${WORK_DIR}/nuget/package" -n "${NUGET_ID}"
run_step nuget-consumer-create dotnet new console --no-restore --force -o "${WORK_DIR}/nuget/consumer" -n TeaQL.Native.Consumer
run_step nuget-pack dotnet pack "${WORK_DIR}/nuget/package" -c Release \
  -p:PackageId="${NUGET_ID}" -p:PackageVersion=0.0.1 -o "${WORK_DIR}/nuget/dist"
printf '<configuration></configuration>\n' >"${WORK_DIR}/nuget/NuGet.Config"
run_step nuget-source dotnet nuget add source "${REGISTRY_URL}/repository/nuget-hosted/v3/index.json" \
  --name teaql-native --username admin --password "${PAT}" --store-password-in-clear-text \
  --configfile "${WORK_DIR}/nuget/NuGet.Config"
run_step nuget-publish bash -c 'cd "$1" && shift && exec "$@"' _ "${WORK_DIR}/nuget" \
  dotnet nuget push "${WORK_DIR}/nuget/dist/${NUGET_ID}.0.0.1.nupkg" \
  --source teaql-native --api-key "${PAT}" --skip-duplicate
run_step nuget-consume dotnet add "${WORK_DIR}/nuget/consumer" package "${NUGET_ID}" \
  --version 0.0.1 --source "${REGISTRY_URL}/repository/nuget-hosted/v3/index.json" \
  --no-restore
run_step nuget-restore dotnet restore "${WORK_DIR}/nuget/consumer" \
  --configfile "${WORK_DIR}/nuget/NuGet.Config"
printf 'NuGet\tPASS\tPASS\n' >>"${SUMMARY}"

# Docker
# The current Docker API maps the first path segment to the configured TeaQL
# repository. Use a unique tag inside the pre-provisioned docker-hosted repo.
DOCKER_IMAGE="${REGISTRY_URL#*://}/docker-hosted:native-probe-${RUN_ID}"
mkdir -p "${WORK_DIR}/docker/context" "${WORK_DIR}/docker/config"
printf 'native docker probe %s\n' "${RUN_ID}" >"${WORK_DIR}/docker/context/probe.txt"
cat >"${WORK_DIR}/docker/context/Dockerfile" <<'EOF'
FROM scratch
COPY probe.txt /probe.txt
EOF
run_step docker-build docker --config "${WORK_DIR}/docker/config" build -t "${DOCKER_IMAGE}" "${WORK_DIR}/docker/context"
printf '%s' "${PAT}" | docker --config "${WORK_DIR}/docker/config" login \
  --username admin --password-stdin "${REGISTRY_URL#*://}" >"${WORK_DIR}/docker-login.raw.log" 2>&1
sanitize_log "${WORK_DIR}/docker-login.raw.log" "${RESULTS_DIR}/docker-login.txt"
run_step docker-publish docker --config "${WORK_DIR}/docker/config" push "${DOCKER_IMAGE}"
docker image rm "${DOCKER_IMAGE}" >/dev/null
run_step docker-consume docker --config "${WORK_DIR}/docker/config" pull "${DOCKER_IMAGE}"
printf 'Docker\tPASS\tPASS\n' >>"${SUMMARY}"

# Raw
printf 'native raw probe %s\n' "${RUN_ID}" >"${WORK_DIR}/raw-source.bin"
run_step raw-publish curl -fsS --user "admin:${PAT}" -X PUT \
  --data-binary "@${WORK_DIR}/raw-source.bin" \
  "${REGISTRY_URL}/repository/raw-hosted/native/${RUN_ID}/probe.bin"
run_step raw-consume curl -fsS --user "admin:${PAT}" \
  -o "${WORK_DIR}/raw-result.bin" \
  "${REGISTRY_URL}/repository/raw-hosted/native/${RUN_ID}/probe.bin"
test "$(sha256sum "${WORK_DIR}/raw-source.bin" | awk '{print $1}')" = \
  "$(sha256sum "${WORK_DIR}/raw-result.bin" | awk '{print $1}')"
printf 'Raw\tPASS\tPASS\n' >>"${SUMMARY}"

echo "Native client evidence written to ${RESULTS_DIR}"
