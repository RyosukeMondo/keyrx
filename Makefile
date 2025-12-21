# KeyRx2 Makefile
# Provides simple top-level commands for common operations

.PHONY: help build verify test launch clean setup

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

launch: ## Launch the keyrx_daemon
	@scripts/launch.sh

clean: ## Remove build artifacts and logs
	@echo "Cleaning build artifacts..."
	@rm -rf target/
	@rm -rf keyrx_ui/node_modules/
	@rm -rf keyrx_ui/dist/
	@rm -rf keyrx_daemon/ui_dist/
	@rm -f scripts/logs/*.log
	@rm -rf .vite/
	@echo "Clean complete."

setup: ## Install development tools and git hooks
	@echo "Installing development tools..."
	@command -v cargo-watch >/dev/null 2>&1 || cargo install cargo-watch
	@command -v cargo-tarpaulin >/dev/null 2>&1 || cargo install cargo-tarpaulin
	@command -v cargo-fuzz >/dev/null 2>&1 || cargo install cargo-fuzz
	@command -v wasm-pack >/dev/null 2>&1 || cargo install wasm-pack
	@echo "Installing BATS testing framework..."
	@if ! command -v bats >/dev/null 2>&1; then \
		if command -v apt-get >/dev/null 2>&1; then \
			echo "Installing BATS via apt-get (requires sudo)..."; \
			sudo apt-get update && sudo apt-get install -y bats; \
		elif command -v brew >/dev/null 2>&1; then \
			echo "Installing BATS via Homebrew..."; \
			brew install bats-core; \
		else \
			echo "Warning: BATS not found and no package manager detected."; \
			echo "Please install BATS manually: https://github.com/bats-core/bats-core"; \
		fi; \
	fi
	@echo "Installing git hooks..."
	@scripts/setup_hooks.sh
	@echo "Setup complete."
