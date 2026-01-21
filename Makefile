# KeyRx2 Makefile
# Provides simple top-level commands for common operations

.PHONY: help build verify test launch clean setup msi e2e-auto

# Default target - show help
.DEFAULT_GOAL := help

help: ## Show this help message
	@echo "KeyRx2 Development Commands"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  %-12s %s\n", $$1, $$2}'
	@echo ""

build: ## Build all workspace crates
	@scripts/build.sh

verify: ## Run all quality checks (clippy, fmt, tests, coverage)
	@scripts/verify.sh

test: ## Run all tests
	@scripts/test.sh

test-fast: ## Run tests with nextest (faster, parallel execution)
	@scripts/test.sh --nextest

e2e-auto: build ## Run automated E2E API tests with auto-fix
	@echo "Running automated E2E API tests..."
	@cd keyrx_ui && npm run test:e2e:auto
	@echo "E2E tests complete. Generating HTML report..."
	@cd keyrx_ui && npm run test:e2e:auto:report
	@echo "HTML report generated: keyrx_ui/report.html"

launch: ## Launch the keyrx_daemon
	@scripts/launch.sh

clean: ## Remove build artifacts and logs
	@echo "Cleaning build artifacts..."
	@rm -rf target/
	@rm -rf keyrx_ui/node_modules/
	@rm -rf keyrx_ui/dist/
	@rm -rf keyrx_ui_v2/node_modules/
	@rm -rf keyrx_ui_v2/dist/
	@rm -rf keyrx_ui_v2/src/wasm/pkg/
	@rm -rf keyrx_daemon/ui_dist/
	@rm -f scripts/logs/*.log
	@rm -rf .vite/
	@echo "Clean complete."

setup: ## Install development tools and git hooks (comprehensive setup)
	@scripts/setup_dev_environment.sh

setup-quick: ## Quick setup (no sudo, Cargo tools only)
	@echo "Quick setup: Installing Cargo development tools..."
	@command -v cargo-watch >/dev/null 2>&1 || cargo install cargo-watch
	@command -v cargo-llvm-cov >/dev/null 2>&1 || cargo install cargo-llvm-cov
	@command -v cargo-fuzz >/dev/null 2>&1 || cargo install cargo-fuzz
	@command -v cargo-nextest >/dev/null 2>&1 || cargo install cargo-nextest --locked
	@command -v wasm-pack >/dev/null 2>&1 || cargo install wasm-pack
	@echo "Installing git hooks..."
	@scripts/setup_hooks.sh
	@echo "Quick setup complete. For full setup (with system dependencies), run: make setup"

msi: ## Build Windows MSI installer (Windows only, requires WiX)
ifeq ($(OS),Windows_NT)
	@cmd /c scripts\windows\build_msi.bat
else
	@echo "MSI build is only supported on Windows"
	@echo "Use: scripts/windows/build_msi.bat (from cmd/PowerShell)"
endif
