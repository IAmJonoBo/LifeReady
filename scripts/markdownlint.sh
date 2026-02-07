#!/usr/bin/env bash
set -euo pipefail

TOOLS_DIR="tools/markdownlint"

if [[ ! -f "${TOOLS_DIR}/package.json" ]]; then
	echo "ERR: markdownlint tools package.json not found: ${TOOLS_DIR}/package.json" >&2
	exit 2
fi

if [[ ! -x "${TOOLS_DIR}/node_modules/.bin/markdownlint" ]]; then
	echo "ERR: markdownlint-cli not installed (would exit 127)." >&2
	echo "Install: npm install --prefix tools/markdownlint" >&2
	exit 127
fi

(cd "${TOOLS_DIR}" && npx markdownlint-cli "**/*.md" --config "../../.markdownlint.yml" --ignore-path "../../.markdownlintignore")
