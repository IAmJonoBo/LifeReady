#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   scripts/generate-axum.sh "<spec1> <spec2> ..." [SERVICE]
#
# Generates into:
#   services/<service>/generated
#
# Service name is derived from spec filename:
#   packages/contracts/<service>.openapi.yaml -> services/<service>/generated

SPECS="${1:-}"
SERVICE_FILTER="${2:-}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS_DIR="${REPO_ROOT}/tools/openapi"
CONFIG_FILE="${OPENAPI_GENERATOR_CONFIG:-${REPO_ROOT}/packages/contracts/openapi-generator.base.yaml}"

if [[ -z "${SPECS}" ]]; then
  echo "ERR: No specs provided." >&2
  echo "Usage: $0 \"packages/contracts/*.openapi.yaml\" [SERVICE]" >&2
  exit 2
fi

if [[ ! -f "${TOOLS_DIR}/package.json" ]]; then
  echo "ERR: OpenAPI tools package.json not found: ${TOOLS_DIR}/package.json" >&2
  echo "Run: npm install --prefix tools/openapi" >&2
  exit 2
fi

if [[ ! -f "${TOOLS_DIR}/openapitools.json" ]]; then
  echo "ERR: OpenAPI Generator version config missing: ${TOOLS_DIR}/openapitools.json" >&2
  exit 2
fi

if [[ ! -x "${TOOLS_DIR}/node_modules/.bin/openapi-generator-cli" ]]; then
  echo "ERR: openapi-generator-cli not installed (would exit 127)." >&2
  echo "Install: npm install --prefix tools/openapi" >&2
  exit 127
fi

if [[ ! -f "${CONFIG_FILE}" ]]; then
  echo "ERR: OpenAPI Generator config not found: ${CONFIG_FILE}" >&2
  exit 2
fi

mapfile -t files < <(eval "ls -1 ${SPECS}" 2>/dev/null || true)
if [[ "${#files[@]}" -eq 0 ]]; then
  echo "ERR: No OpenAPI specs matched: ${SPECS}" >&2
  exit 2
fi

echo "Generating Rust Axum stubs from OpenAPI specs..."

for spec in "${files[@]}"; do
  base="$(basename "${spec}")"
  svc="${base%.openapi.yaml}"

  if [[ -n "${SERVICE_FILTER}" && "${svc}" != "${SERVICE_FILTER}" ]]; then
    continue
  fi

  out_dir="services/${svc}/generated"
  mkdir -p "${out_dir}"

  echo " - ${svc}: ${spec} -> ${out_dir}"

  # Clean output to avoid stale artifacts.
  rm -rf "${out_dir:?}/"*

  # Minimal, stable generator flags.
  # You can later add a config file per service if you want custom templates/package naming.
  (cd "${TOOLS_DIR}" && npx openapi-generator-cli generate \
    -g rust-axum \
    -i "${spec}" \
    -o "${out_dir}" \
    -c "${CONFIG_FILE}" \
    --additional-properties=packageName="${svc//-/_}",crateName="${svc//-/_}")

done

echo "Generation complete."
