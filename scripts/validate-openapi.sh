#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   scripts/validate-openapi.sh "<spec1> <spec2> ..." [SERVICE]
# Examples:
#   scripts/validate-openapi.sh "packages/contracts/*.openapi.yaml"
#   scripts/validate-openapi.sh "packages/contracts/*.openapi.yaml" estate-service

SPECS="${1:-}"
SERVICE_FILTER="${2:-}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS_DIR="${REPO_ROOT}/tools/openapi"

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

# Expand globs safely
mapfile -t files < <(eval "ls -1 ${SPECS}" 2>/dev/null || true)

if [[ "${#files[@]}" -eq 0 ]]; then
  echo "ERR: No OpenAPI specs matched: ${SPECS}" >&2
  exit 2
fi

echo "Validating OpenAPI specs with openapi-generator-cli validate..."

fail=0
for spec in "${files[@]}"; do
  base="$(basename "${spec}")"
  svc="${base%.openapi.yaml}"

  # Optional service filter: SERVICE should match directory name or spec stem.
  if [[ -n "${SERVICE_FILTER}" ]]; then
    if [[ "${svc}" != "${SERVICE_FILTER}" && "${svc}" != "${SERVICE_FILTER%-service}" ]]; then
      if [[ "${svc}" != "${SERVICE_FILTER}" ]]; then
        continue
      fi
    fi
  fi

  echo " - ${spec}"
  if ! (cd "${TOOLS_DIR}" && npx openapi-generator-cli validate -i "${spec}" --recommend); then
    echo "   FAIL: ${spec}" >&2
    fail=1
  fi
done

if [[ "${fail}" -ne 0 ]]; then
  echo "OpenAPI validation failed." >&2
  exit 1
fi

echo "OpenAPI validation OK."
