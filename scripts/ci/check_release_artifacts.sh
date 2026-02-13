#!/usr/bin/env bash
set -euo pipefail

required_files=(
	"docs/adr/README.md"
	"docs/e2e-production-readiness-task-board.md"
	"docs/ops/runbooks.md"
	"docs/releases/go-no-go-checklist.md"
	"docs/qa/staging-e2e-report.md"
)

missing=0
for file in "${required_files[@]}"; do
	if [[ ! -f $file ]]; then
		echo "Missing required release artifact: $file"
		missing=1
	fi
done

if [[ $missing -ne 0 ]]; then
	exit 1
fi

echo "All required release artifacts are present."
