#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
TOOLS_DIR="${repo_root}/tools/markdownlint"

if [[ ! -f "${TOOLS_DIR}/package.json" ]]; then
	echo "ERR: markdownlint tools package.json not found: ${TOOLS_DIR}/package.json" >&2
	exit 2
fi

if [[ ! -x "${TOOLS_DIR}/node_modules/.bin/markdownlint" ]]; then
	echo "ERR: markdownlint-cli not installed (would exit 127)." >&2
	echo "Install: npm install --prefix tools/markdownlint" >&2
	exit 127
fi

(cd "${repo_root}" && "${TOOLS_DIR}/node_modules/.bin/markdownlint" "." --config ".markdownlint.yml" --ignore-path ".markdownlintignore" --dot)
