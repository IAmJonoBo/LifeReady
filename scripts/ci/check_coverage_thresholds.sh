#!/usr/bin/env bash
set -euo pipefail

mode="${LIFEREADY_COVERAGE_MODE:-baseline}"

run_cov() {
	local label="$1"
	shift
	echo "==> Coverage gate: ${label}"
	cargo llvm-cov "$@"
}

if [[ $mode == "target" ]]; then
	run_cov "workspace regions >= 75" \
		--workspace --all-targets --all-features --summary-only --fail-under-regions 75

	run_cov "case-service regions >= 65" \
		-p case_service --all-targets --all-features --summary-only --fail-under-regions 65

	run_cov "vault-service regions >= 70" \
		-p vault_service --all-targets --all-features --summary-only --fail-under-regions 70

	run_cov "audit-service regions >= 70" \
		-p audit_service --all-targets --all-features --summary-only --fail-under-regions 70
else
	run_cov "workspace regions >= 62 (baseline ratchet)" \
		--workspace --all-targets --all-features --summary-only --fail-under-regions 62

	run_cov "case-service regions >= 31 (baseline ratchet)" \
		-p case_service --all-targets --all-features --summary-only --fail-under-regions 31

	run_cov "vault-service regions >= 43 (baseline ratchet)" \
		-p vault_service --all-targets --all-features --summary-only --fail-under-regions 43

	run_cov "audit-service regions >= 53 (baseline ratchet)" \
		-p audit_service --all-targets --all-features --summary-only --fail-under-regions 53
fi

if [[ $mode == "target" ]]; then
	run_cov "lifeready-auth lines >= 90" \
		-p lifeready-auth --all-targets --all-features --summary-only --fail-under-lines 90

	run_cov "lifeready-policy lines >= 90" \
		-p lifeready-policy --all-targets --all-features --summary-only --fail-under-lines 90
else
	run_cov "lifeready-auth lines >= 83 (baseline ratchet)" \
		-p lifeready-auth --all-targets --all-features --summary-only --fail-under-lines 83

	run_cov "lifeready-policy lines >= 95 (baseline ratchet)" \
		-p lifeready-policy --all-targets --all-features --summary-only --fail-under-lines 95
fi

echo "All coverage thresholds passed."
