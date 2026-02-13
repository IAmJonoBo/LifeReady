#!/usr/bin/env bash
set -euo pipefail

services=(
	"identity-service"
	"estate-service"
	"vault-service"
	"case-service"
	"audit-service"
)

for service in "${services[@]}"; do
	lib="services/${service}/src/lib.rs"
	contract="packages/contracts/${service}.openapi.yaml"
	if ! grep -q 'route("/readyz"' "$lib"; then
		echo "Missing /readyz route in ${lib}"
		exit 1
	fi
	if ! grep -q '^  /readyz:$' "$contract"; then
		echo "Missing /readyz path in ${contract}"
		exit 1
	fi
	if ! grep -q '"200":' "$contract" || ! grep -q 'components/responses/Ready"' "$contract"; then
		echo "Missing 200 Ready response for /readyz in ${contract}"
		exit 1
	fi
	if ! grep -q '"503":' "$contract" || ! grep -q 'components/responses/NotReady"' "$contract"; then
		echo "Missing 503 NotReady response for /readyz in ${contract}"
		exit 1
	fi
done

grep -q '/readyz' services/identity-service/tests/smoke.rs
grep -q '/readyz' services/estate-service/tests/smoke.rs
grep -q '/readyz' services/vault-service/tests/smoke.rs
grep -q '/readyz' services/case-service/tests/smoke.rs
grep -q '/readyz' services/audit-service/tests/contract.rs

echo "Readiness endpoint routes and tests are present for all services."
