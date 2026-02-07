#!/usr/bin/env bash
set -euo pipefail

if ! command -v dart >/dev/null 2>&1; then
	echo "ERR: dart not found. Install Flutter or Dart SDK." >&2
	exit 127
fi

if [[ ! -f "apps/lifeready_flutter/tool/generate_tokens.dart" ]]; then
	echo "ERR: tokens generator not found." >&2
	exit 2
fi

(cd apps/lifeready_flutter && dart run tool/generate_tokens.dart)
