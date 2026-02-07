SHELL := /usr/bin/env bash
.SHELLFLAGS := -euo pipefail -c

CONTRACTS_DIR := packages/contracts
SCRIPTS_DIR := scripts

OPENAPI_SPECS := $(wildcard $(CONTRACTS_DIR)/*.openapi.yaml)
SERVICES := $(patsubst %.openapi.yaml,%,$(notdir $(OPENAPI_SPECS)))

.PHONY: help
help:
	@echo "Targets:"
	@echo "  make validate-openapi     Validate all OpenAPI specs in $(CONTRACTS_DIR)"
	@echo "  make generate-axum        Generate Rust Axum stubs into services/*/generated"
	@echo "  make validate-openapi-<service>  Validate a single service spec"
	@echo "  make generate-axum-<service>     Generate stubs for a single service"
	@echo "  make generate-flutter-tokens    Regenerate Flutter tokens from DTCG JSON"
	@echo "  make flutter-check              Run tokens + analyze + test for Flutter"
	@echo "  make lint-docs                  Run markdownlint on docs"
	@echo "  make upgrade-toolchain          Install/refresh repo toolchains"
	@echo "  make dev-up                     Start local dev services"
	@echo "  make dev-down                   Stop local dev services"
	@echo "  make db-migrate                 Run service migrations"
	@echo "  make clean-generated      Remove generated outputs"
	@echo ""
	@echo "Options:"
	@echo "  SERVICE=<name>            Limit generation/validation to one service (e.g. SERVICE=estate-service)"
	@echo "  SPECS=<glob>              Override spec selection (e.g. SPECS='packages/contracts/*service.openapi.yaml')"

.PHONY: validate-openapi
validate-openapi:
	@$(SCRIPTS_DIR)/validate-openapi.sh "$(or $(SPECS),$(OPENAPI_SPECS))" $(SERVICE)

.PHONY: validate-openapi-%
validate-openapi-%:
	@$(SCRIPTS_DIR)/validate-openapi.sh $(CONTRACTS_DIR)/$*.openapi.yaml $*

.PHONY: generate-axum
generate-axum:
	@$(SCRIPTS_DIR)/generate-axum.sh "$(or $(SPECS),$(OPENAPI_SPECS))" $(SERVICE)

.PHONY: generate-axum-%
generate-axum-%:
	@$(SCRIPTS_DIR)/generate-axum.sh $(CONTRACTS_DIR)/$*.openapi.yaml $*

.PHONY: clean-generated
clean-generated:
	@echo "Cleaning services/*/generated ..."
	@rm -rf services/*/generated

.PHONY: generate-flutter-tokens
generate-flutter-tokens:
	@$(SCRIPTS_DIR)/generate-flutter-tokens.sh

.PHONY: flutter-check
flutter-check: generate-flutter-tokens
	@cd apps/lifeready_flutter && flutter pub get
	@cd apps/lifeready_flutter && flutter analyze
	@cd apps/lifeready_flutter && if [ -d test ]; then flutter test; else echo "No Flutter tests yet."; fi

.PHONY: lint-docs
lint-docs:
	@$(SCRIPTS_DIR)/markdownlint.sh

.PHONY: upgrade-toolchain
upgrade-toolchain:
	@$(SCRIPTS_DIR)/upgrade-toolchain.sh

.PHONY: dev-up
dev-up:
	docker compose up -d

.PHONY: dev-down
dev-down:
	docker compose down -v

.PHONY: db-migrate
db-migrate:
	@echo "Running migrations for services with migrations/..."
	@for svc in audit-service estate-service vault-service case-service ; do \
	  if [ -d services/$$svc/migrations ]; then \
	    echo " - $$svc"; \
	    sqlx migrate run --source services/$$svc/migrations ; \
	  fi \
	done
