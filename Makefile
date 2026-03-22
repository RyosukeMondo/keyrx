# KeyRx Makefile
#
# IMPORTANT: Always use 'make build' (not bare 'cargo build') to get a
# correct binary with fresh WASM + UI embedded. The daemon build.rs will
# FAIL if it detects stale frontend artifacts.
#
# Build order: WASM → UI → Daemon (enforced by scripts/build.sh)
#
# Bypass frontend checks for Rust-only dev:
#   KEYRX_SKIP_FRONTEND_CHECK=1 cargo build -p keyrx_daemon

.PHONY: help build build-release verify test test-fast launch clean setup \
        sync-version e2e-auto installer installer-simple msi

.DEFAULT_GOAL := help

help: ## Show available targets
	@echo "KeyRx Development Commands"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Build targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  %-20s %s\n", $$1, $$2}'
	@echo ""
	@echo "NOTE: Always use 'make build', not bare 'cargo build'."
	@echo "      The build enforces WASM -> UI -> Daemon order."

# ── Build ─────────────────────────────────────────────────────────────

build: ## Full build: WASM -> UI -> Daemon (THE correct entry point)
	@scripts/build.sh

build-release: ## Full release build (optimized, LTO)
	@scripts/build.sh --release

# ── Quality ───────────────────────────────────────────────────────────

verify: ## Run all quality checks (clippy, fmt, tests, coverage)
	@scripts/verify.sh

test: ## Run all tests
	@scripts/test.sh

test-fast: ## Run tests with nextest (faster parallel execution)
	@scripts/test.sh --nextest

# ── Run ───────────────────────────────────────────────────────────────

launch: ## Build and launch daemon
	@scripts/launch.sh

# ── Maintenance ───────────────────────────────────────────────────────

clean: ## Remove ALL build artifacts (target, dist, wasm, node_modules)
	@echo "Cleaning all build artifacts..."
	@rm -rf target/
	@rm -rf keyrx_ui/dist/
	@rm -rf keyrx_ui/src/wasm/pkg/
	@rm -rf keyrx_ui/node_modules/
	@rm -rf installer-output/
	@rm -f scripts/logs/*.log
	@rm -rf .vite/
	@echo "Clean complete."

setup: ## Install development tools and git hooks
	@scripts/setup.sh

sync-version: ## Sync version from Cargo.toml (SSOT) to all files
	@scripts/sync-version.sh

# ── E2E & Packaging ──────────────────────────────────────────────────

e2e-auto: build ## Run automated E2E tests (builds first)
	@cd keyrx_ui && npm run test:e2e:auto

installer: build-release ## Build NSIS installer (full release build + package)
ifeq ($(OS),Windows_NT)
	$(eval VERSION := $(shell grep -A5 '^\[workspace.package\]' Cargo.toml | grep '^version' | head -1 | sed 's/.*"\(.*\)".*/\1/'))
	@echo "Building NSIS installer v$(VERSION)..."
	@mkdir -p installer-output
	@MSYS_NO_PATHCONV=1 "/c/Program Files (x86)/NSIS/makensis.exe" /DVERSION=$(VERSION) scripts/nsis/keyrx-installer.nsi
	@echo "Installer: installer-output/keyrx-setup-v$(VERSION)-windows-x64.exe"
else
	@echo "Installer build is only supported on Windows"
endif

installer-simple: ## Build PowerShell installer (Windows only)
ifeq ($(OS),Windows_NT)
	@powershell.exe -ExecutionPolicy Bypass -File scripts\installer\create-simple-installer.ps1
else
	@echo "Installer build is only supported on Windows"
endif

msi: ## Build MSI installer (Windows only, requires WiX)
ifeq ($(OS),Windows_NT)
	@cmd /c scripts\windows\build_msi.bat
else
	@echo "MSI build is only supported on Windows"
endif
